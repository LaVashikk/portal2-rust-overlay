use bitflags::bitflags;

bitflags! {
    /// Flags representing the contents of a leaf or brush.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ContentsFlags: i32 {
        const EMPTY                = 0;
        const SOLID                = 0x1;
        const WINDOW               = 0x2;
        const AUX                  = 0x4;
        const GRATE                = 0x8;
        const SLIME                = 0x10;
        const WATER                = 0x20;
        const BLOCKLOS             = 0x40;
        const OPAQUE               = 0x80;
        const TESTFOGVOLUME        = 0x100;
        const UNUSED               = 0x200;
        const BLOCKLIGHT           = 0x400;
        const TEAM1                = 0x800;
        const TEAM2                = 0x1000;
        const IGNORE_NODRAW_OPAQUE = 0x2000;
        const MOVEABLE             = 0x4000;
        const AREAPORTAL           = 0x8000;
        const PLAYERCLIP           = 0x10000;
        const MONSTERCLIP          = 0x20000;
        const BRUSH_PAINT          = 0x40000;
        const GRENADECLIP          = 0x80000;
        const ORIGIN               = 0x1000000;
        const MONSTER              = 0x2000000;
        const DEBRIS               = 0x4000000;
        const DETAIL               = 0x8000000;
        const TRANSLUCENT          = 0x10000000;
        const LADDER               = 0x20000000;
        const HITBOX               = 0x40000000;
    }
}

bitflags! {
    /// Predefined masks for spatial queries (traces).
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MaskFlags: i32 {
        const ALL           = -1; // 0xFFFFFFFF
        
        /// everything that is normally solid
        const SOLID         = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::WINDOW.bits() | ContentsFlags::MONSTER.bits() | 
                             ContentsFlags::GRATE.bits();

        /// everything that blocks player movement
        const PLAYERSOLID   = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::PLAYERCLIP.bits() | ContentsFlags::WINDOW.bits() | 
                             ContentsFlags::MONSTER.bits() | ContentsFlags::GRATE.bits();

        /// blocks npc movement
        const NPCSOLID      = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::MONSTERCLIP.bits() | ContentsFlags::WINDOW.bits() | 
                             ContentsFlags::MONSTER.bits() | ContentsFlags::GRATE.bits();

        /// everything that blocks lighting
        const OPAQUE        = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::OPAQUE.bits();

        /// everything that blocks line of sight for players
        const VISIBLE       = Self::OPAQUE.bits() | ContentsFlags::IGNORE_NODRAW_OPAQUE.bits();

        /// bullets see these as solid
        const SHOT          = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::MONSTER.bits() | ContentsFlags::WINDOW.bits() | 
                             ContentsFlags::DEBRIS.bits() | ContentsFlags::HITBOX.bits();

        /// non-raycasted weapons see this as solid (includes grates)
        const SHOT_HULL     = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::MONSTER.bits() | ContentsFlags::WINDOW.bits() | 
                             ContentsFlags::DEBRIS.bits() | ContentsFlags::GRATE.bits();
                             
        /// for finding floor height
        const FLOORTRACE    = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                             ContentsFlags::WINDOW.bits() | ContentsFlags::DEBRIS.bits();
    }
}
