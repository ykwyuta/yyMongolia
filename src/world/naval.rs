//! 船：兵を乗せて海を渡る。乗る（Board）・降ろす（Unload）

use super::*;

impl World {
    /// 兵を船に乗せる命令（船の定員まで。船は岸の近くにいなければならない）
    pub fn order_board(&mut self, hs: &[Handle], ship: Handle) {
        self.note_player_order(hs);
        let Some(s) = self.unit(ship) else { return };
        if !s.kind.naval() {
            return;
        }
        let sn = s.nation;
        for &h in hs {
            if h == ship {
                continue;
            }
            if let Some(u) = self.unit_mut(h) {
                if u.nation == sn && !u.kind.naval() && u.aboard.is_none() {
                    u.order = Order::Board(ship);
                    u.resume = None;
                    u.build_queue.clear();
                    u.reset_nav();
                }
            }
        }
    }

    /// 船に、この地点の近くの岸で兵を降ろすよう命じる
    pub fn order_unload(&mut self, hs: &[Handle], target: V2) {
        self.note_player_order(hs);
        for &h in hs {
            if let Some(u) = self.unit_mut(h) {
                if u.kind.naval() {
                    u.order = Order::Unload(target);
                    u.resume = None;
                    u.reset_nav();
                }
            }
        }
    }

    /// 船の上の兵を、いまいる岸辺にすぐ降ろす（陸が近くになければ false）
    pub fn unload_here(&mut self, sh: Handle, then: Option<V2>) -> bool {
        let Some(s) = self.unit(sh) else { return false };
        if s.cargo.is_empty() {
            return false;
        }
        let pos = s.pos;
        let spots = self.land_near(pos, 2);
        if spots.is_empty() {
            return false;
        }
        let cargo = std::mem::take(&mut self.unit_mut(sh).unwrap().cargo);
        for (k, h) in cargo.into_iter().enumerate() {
            let p = spots[k % spots.len()];
            if let Some(u) = self.unit_mut(h) {
                u.aboard = None;
                u.pos = p;
                u.reset_nav();
                u.order = match then {
                    Some(g) if u.kind.military() => Order::AttackMove(g),
                    Some(g) => Order::Move(g),
                    None => Order::Idle,
                };
            }
        }
        true
    }

    pub(super) fn update_board(&mut self, i: usize, sh: Handle, dt: f32) {
        let ni = self.units[i].nation;
        let ok = self.unit(sh).map_or(false, |s| s.alive && s.nation == ni && s.kind.naval() && s.cargo.len() < SHIP_CAP);
        if !ok {
            self.units[i].order = Order::Idle;
            return;
        }
        let sp = self.unit(sh).unwrap().pos;
        let pos = self.units[i].pos;
        if pos.dist(sp) <= 1.9 {
            let me = self.uh(i);
            let u = &mut self.units[i];
            u.aboard = Some(sh);
            u.order = Order::Idle;
            u.selected = false;
            u.path.clear();
            u.carry = 0.0;
            self.unit_mut(sh).unwrap().cargo.push(me);
            return;
        }
        // 船のそばの陸へ
        let goal = self.land_near(sp, 1).first().copied().unwrap_or(sp);
        if self.nav(i, goal, 0.5, dt) && self.units[i].pos.dist(sp) > 1.9 {
            self.units[i].order = Order::Idle;
        }
    }

    pub(super) fn update_unload(&mut self, i: usize, target: V2, dt: f32) {
        if self.units[i].cargo.is_empty() {
            self.units[i].order = Order::Idle;
            self.units[i].resume = None;
            return;
        }
        // 着ける岸辺（一度だけ求めて覚えておく）
        let landing = match self.units[i].resume {
            Some(p) => p,
            None => {
                let sea = self.sea_region[{
                    let (x, y) = self.units[i].pos.tile();
                    Map::idx(x.clamp(0, MAP_W as i32 - 1), y.clamp(0, MAP_H as i32 - 1))
                }];
                let mut best = None;
                for r in [6, 12, 20] {
                    if let Some(p) = self.shore_water_near(target, r) {
                        let (x, y) = p.tile();
                        if self.sea_region[Map::idx(x, y)] == sea {
                            best = Some(p);
                            break;
                        }
                    }
                }
                match best {
                    Some(p) => {
                        self.units[i].resume = Some(p);
                        p
                    }
                    None => {
                        self.units[i].order = Order::Idle;
                        return;
                    }
                }
            }
        };
        let pos = self.units[i].pos;
        let near_land = !self.land_near(pos, 1).is_empty();
        if (pos.dist(landing) <= 1.2 || (pos.dist(landing) < 4.0 && near_land)) && near_land {
            let me = self.uh(i);
            self.unload_here(me, Some(target));
            self.units[i].order = Order::Idle;
            self.units[i].resume = None;
            return;
        }
        if self.nav(i, landing, 0.6, dt) {
            let me = self.uh(i);
            if !self.unload_here(me, Some(target)) {
                self.units[i].order = Order::Idle;
            }
            self.units[i].order = Order::Idle;
            self.units[i].resume = None;
        }
    }
}
