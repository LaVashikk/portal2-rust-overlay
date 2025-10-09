use bitflags::bitflags;

bitflags! {
    /// Represents the bitmask of flags for a ConVar.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CvarFlags: i32 {
        /// The default, no flags at all.
        const NONE = 0;
        /// If this is set, don't add to linked list, etc. It's a ConVar that is not registered.
        const UNREGISTERED = 1 << 0;
        /// Hidden in released products. Flag is removed automatically if ALLOW_DEVELOPMENT_CVARS is defined.
        const DEVELOPMENTONLY = 1 << 1;
        /// Defined by the game DLL.
        const GAMEDLL = 1 << 2;
        /// Defined by the client DLL.
        const CLIENTDLL = 1 << 3;
        /// Hidden. Doesn't appear in find or autocomplete. Like DEVELOPMENTONLY, but can't be compiled out.
        const HIDDEN = 1 << 4;

        /// It's a server cvar, but we don't send the data since it's a password, etc. Sends 1 if it's not bland/zero, 0 otherwise as value.
        const PROTECTED = 1 << 5;
        /// This cvar cannot be changed by clients connected to a multiplayer server.
        const SPONLY = 1 << 6;
        /// Set to cause it to be saved to config.cfg.
        const ARCHIVE = 1 << 7;
        /// Notifies players when changed.
        const NOTIFY = 1 << 8;
        /// Changes the client's info string.
        const USERINFO = 1 << 9;
        /// This cvar's string cannot contain unprintable characters (e.g., used for player name).
        const PRINTABLEONLY = 1 << 10;
        /// If this is a FCVAR_SERVER, don't log changes to the log file / console if we are creating a log.
        const UNLOGGED = 1 << 11;
        /// Never try to print that cvar. If this is set, don't use the string value of the cvar.
        const NEVER_AS_STRING = 1 << 12;

        /// It's a ConVar that's shared between the client and the server.
        /// At signon, the values of all such ConVars are sent from the server to the client.
        /// If a change is requested it must come from the console.
        /// If a value is changed while a server is active, it's replicated to all connected clients.
        const REPLICATED = 1 << 13;
        /// Only useable in singleplayer / debug / multiplayer & sv_cheats.
        const CHEAT = 1 << 14;
        /// This var isn't archived, but is exposed to players—and its use is allowed in competitive play.
        const INTERNAL_USE = 1 << 15;
        /// Record this cvar when starting a demo file.
        const DEMO = 1 << 16;
        /// Don't record these commands in demofiles.
        const DONTRECORD = 1 << 17;
        /// This convar can be changed in competitive (strict) settings mode even though it is not archived.
        const ALLOWED_IN_COMPETITIVE = 1 << 18;
        /// Cvar is release.
        const RELEASE = 1 << 19;
        /// If this cvar changes, it forces a material reload.
        const RELOAD_MATERIALS = 1 << 20;
        /// If this cvar changes, it forces a texture reload.
        const RELOAD_TEXTURES = 1 << 21;

        /// Cvar cannot be changed by a client that is connected to a server.
        const NOT_CONNECTED = 1 << 22;
        /// Indicates this cvar is read from the material system thread.
        const MATERIAL_SYSTEM_THREAD = 1 << 23;
        /// Cvar written to config.cfg on the Xbox.
        const ARCHIVE_GAMECONSOLE = 1 << 24;
        /// Used as a debugging tool necessary to check material system thread convars.
        const ACCESSIBLE_FROM_THREADS = 1 << 25;
        /// The server is allowed to execute this command on clients.
        const SERVER_CAN_EXECUTE = 1 << 28;
        /// If this is set, then the server is not allowed to query this cvar's value.
        const SERVER_CANNOT_QUERY = 1 << 29;
        /// IVEngineClient::ClientCmd is allowed to execute this command.
        const CLIENTCMD_CAN_EXECUTE = 1 << 30;
    }
}
