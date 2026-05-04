use bitflags::bitflags;

bitflags! {
    /// Flags representing the contents of a leaf or brush.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ContentsFlags: i32 {
        const EMPTY                = 0;           // No contents
        const SOLID                = 0x1;         // An eye is never valid in a solid
        const WINDOW               = 0x2;         // Translucent, but not watery (glass)
        const AUX                  = 0x4;
        const GRATE                = 0x8;         // Bullets/sight pass through, but solids don't
        const SLIME                = 0x10;
        const WATER                = 0x20;
        const BLOCKLOS             = 0x40;        // Block AI line of sight
        const OPAQUE               = 0x80;        // Things that cannot be seen through
        const TESTFOGVOLUME        = 0x100;
        const UNUSED               = 0x200;
        const BLOCKLIGHT           = 0x400;
        const TEAM1                = 0x800;       // Per team contents
        const TEAM2                = 0x1000;
        const IGNORE_NODRAW_OPAQUE = 0x2000;      // Ignore CONTENTS_OPAQUE on surfaces with SURF_NODRAW
        const MOVEABLE             = 0x4000;      // Hits MOVETYPE_PUSH (doors, plats, etc.)
        const AREAPORTAL           = 0x8000;
        const PLAYERCLIP           = 0x10000;
        const MONSTERCLIP          = 0x20000;
        const BRUSH_PAINT          = 0x40000;     // Portal 2 specific: paintable surface
        const GRENADECLIP          = 0x80000;
        const ORIGIN               = 0x1000000;   // Removed before bsping
        const MONSTER              = 0x2000000;   // In-game only
        const DEBRIS               = 0x4000000;
        const DETAIL               = 0x8000000;   // Brushes to be added after vis leafs
        const TRANSLUCENT          = 0x10000000;  // Auto set if any surface has trans
        const LADDER               = 0x20000000;
        const HITBOX               = 0x40000000;  // Use accurate hitboxes on trace
    }
}

bitflags! {
    /// Predefined masks for spatial queries (traces).
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MaskFlags: i32 {
        const ALL                   = -1; // 0xFFFFFFFF
        
        /// Everything that is normally solid
        const SOLID                 = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::WINDOW.bits() | ContentsFlags::MONSTER.bits() | 
                                     ContentsFlags::GRATE.bits();

        /// Everything that blocks player movement
        const PLAYERSOLID           = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::PLAYERCLIP.bits() | ContentsFlags::WINDOW.bits() | 
                                     ContentsFlags::MONSTER.bits() | ContentsFlags::GRATE.bits();

        /// Blocks npc movement
        const NPCSOLID              = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::MONSTERCLIP.bits() | ContentsFlags::WINDOW.bits() | 
                                     ContentsFlags::MONSTER.bits() | ContentsFlags::GRATE.bits();

        /// Everything that blocks lighting
        const OPAQUE                = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::OPAQUE.bits();

        /// Everything that blocks line of sight for players
        const VISIBLE               = Self::OPAQUE.bits() | ContentsFlags::IGNORE_NODRAW_OPAQUE.bits();

        /// Everything that blocks line of sight for AI
        const BLOCKLOS              = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::BLOCKLOS.bits();

        /// Bullets see these as solid
        const SHOT                  = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::MONSTER.bits() | ContentsFlags::WINDOW.bits() | 
                                     ContentsFlags::DEBRIS.bits() | ContentsFlags::HITBOX.bits();

        /// Non-raycasted weapons see this as solid (includes grates)
        const SHOT_HULL             = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::MONSTER.bits() | ContentsFlags::WINDOW.bits() | 
                                     ContentsFlags::DEBRIS.bits() | ContentsFlags::GRATE.bits();

        /// Portal 2: What blocks portal placement/shots
        const SHOT_PORTAL           = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::WINDOW.bits() | ContentsFlags::MONSTER.bits();
                             
        /// For finding floor height
        const FLOORTRACE            = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::WINDOW.bits() | ContentsFlags::DEBRIS.bits();

        /// Blocks fluid movement
        const NPCFLUID              = ContentsFlags::SOLID.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::MONSTERCLIP.bits() | ContentsFlags::WINDOW.bits() | 
                                     ContentsFlags::MONSTER.bits();

        /// Water physics in these contents
        const WATER                 = ContentsFlags::WATER.bits() | ContentsFlags::MOVEABLE.bits() | 
                                     ContentsFlags::SLIME.bits();
    }
}

bitflags! {
    /// Flags that can be set on a surface (csurface_t).
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SurfaceFlags: u16 {
        const NONE          = 0;
        const LIGHT         = 0x0001; // value will hold the light strength
        const SKY2D         = 0x0002; // don't draw, indicates we should skylight + draw 2d sky but not draw the 3D skybox
        const SKY           = 0x0004; // don't draw, but add to skybox
        const WARP          = 0x0008; // turbulent water warp
        const TRANS         = 0x0010;
        const NOPORTAL      = 0x0020; // the surface can not have a portal placed on it
        const TRIGGER       = 0x0040; // xbox hack to work around elimination of trigger surfaces
        const NODRAW        = 0x0080; // don't bother referencing the texture
        const HINT          = 0x0100; // make a primary bsp splitter
        const SKIP          = 0x0200; // completely ignore, allowing non-closed brushes
        const NOLIGHT       = 0x0400; // Don't calculate light
        const BUMPLIGHT     = 0x0800; // calculate three lightmaps for the surface for bumpmapping
        const NOSHADOWS     = 0x1000; // Don't receive shadows
        const NODECALS      = 0x2000; // Don't receive decals
        const NOPAINT       = 0x2000; // Portal 2: the surface can not have paint placed on it (same as NODECALS)
        const NOCHOP        = 0x4000; // Don't subdivide patches on this surface
        const HITBOX        = 0x8000; // surface is part of a hitbox
    }
}
