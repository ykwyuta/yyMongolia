//! Generated period artwork. Slots are normalized at build-authoring time, never at runtime.
use super::*;

const SIZE: usize = 192;
const CELL: usize = SIZE * SIZE * 4;
const ATLASES: [&[u8]; 10] = [
    include_bytes!("../../assets/period/steppe.bgra"),
    include_bytes!("../../assets/period/song.bgra"),
    include_bytes!("../../assets/period/japan.bgra"),
    include_bytes!("../../assets/period/goryeo.bgra"),
    include_bytes!("../../assets/period/islamic.bgra"),
    include_bytes!("../../assets/period/latin.bgra"),
    include_bytes!("../../assets/period/orthodox.bgra"),
    include_bytes!("../../assets/period/southern.bgra"),
    include_bytes!("../../assets/period/tibet.bgra"),
    include_bytes!("../../assets/period/ainu.bgra"),
];
const EXTRA: &[u8] = include_bytes!("../../assets/period/extra.bgra");
const HEROES_RAW: &[u8] = include_bytes!("../../assets/period/heroes.bgra");

/// Nation order is the simulation's NATIONS/ROSTERS order. Architecture follows origin.
pub fn culture(n: usize) -> usize {
    const MAP: [usize; 34] = [0,1,1,1,1,3,2,7,7,8,0,0,0,0,0,1,9,4,4,4,4,4,5,4,6,6,0,4,6,6,6,5,5,1];
    MAP.get(n).copied().unwrap_or(0)
}

#[derive(Default)]
pub(super) struct PeriodArt {
    normal: Vec<Option<ID2D1Bitmap>>,
    flipped: Vec<Option<ID2D1Bitmap>>,
    fog: Vec<Option<ID2D1Bitmap>>,
    heroes: Vec<Option<ID2D1Bitmap>>,
    heroes_flip: Vec<Option<ID2D1Bitmap>>,
}

impl PeriodArt {
    pub(super) fn load(&mut self, gfx: &Gfx) {
        self.normal.clear(); self.flipped.clear(); self.fog.clear();
        self.heroes.clear(); self.heroes_flip.clear();
        for raw in ATLASES.into_iter().flat_map(|a| a.chunks_exact(CELL)).chain(EXTRA.chunks_exact(CELL)) {
            self.normal.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 1.0)));
            self.flipped.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, true, 1.0)));
            self.fog.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 0.52)));
        }
        for raw in HEROES_RAW.chunks_exact(128 * 128 * 4) {
            self.heroes.push(gfx.create_bitmap(128, 128, &load_raw_sprite_bgra(raw, 128, false, 1.0)));
            self.heroes_flip.push(gfx.create_bitmap(128, 128, &load_raw_sprite_bgra(raw, 128, true, 1.0)));
        }
    }
    pub(super) fn get(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.fog } else if flip { &self.flipped } else { &self.normal }).get(idx).and_then(Option::as_ref)
    }
}

impl Art {
    pub fn national_unit(&self, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        use crate::world::udef;
        // Unique troops retain their equipment even when recruited by a subject's overlord.
        let c = match ut {
            4..=6 | 35 => 0, 8..=17 => 1, 18..=19 => 3, 20..=23 => 2,
            24..=25 => 7, 26 => 8, 27..=31 => 4, 32..=34 => 5,
            36..=39 => 6, 40 => 9, _ => culture(nation),
        };
        let extra = match ut { 10 => Some(0), 11 => Some(1), 19 => Some(2), 23 => Some(3), 16 => Some(4), _ => None };
        if let Some(e) = extra { return self.period.get(120 + e, flip, false); }
        let slot = match ut {
            6 | 9 | 18 | 24 | 28 | 38 | 40 => 5,
            4 | 22 | 29 | 35 => 3,
            _ => match udef(ut).class { UnitKind::Worker => 0, UnitKind::Infantry => 1, UnitKind::Archer => 2, UnitKind::Cavalry => 4, UnitKind::Ship => return self.period.get(120, flip, false) },
        };
        self.period.get(c * 12 + slot, flip, false)
    }
    pub fn national_building(&self, origin: usize, kind: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        let slot = match kind { BKind::Capital => 6, BKind::Town => 7, BKind::Barracks => 8, BKind::Farm => 9, BKind::Fort => 10, BKind::Port => 11 };
        self.period.get(culture(origin) * 12 + slot, false, fog)
    }
    /// The hero's homeland determines dress, independently of the summoning nation.
    pub fn period_hero(&self, hero: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        if hero >= crate::world::HEROES.len() { return None; }
        (if flip { &self.period.heroes_flip } else { &self.period.heroes }).get(hero).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_layout_and_nation_coverage() {
        for a in ATLASES { assert_eq!(a.len(), CELL * 12); }
        assert_eq!(EXTRA.len(), CELL * 8);
        assert_eq!(HEROES_RAW.len(), crate::world::HEROES.len() * 128 * 128 * 4);
        for n in 0..crate::geo::NATIONS.len() { assert!(culture(n) < ATLASES.len()); }
    }
}
