# P2:CE Support Notice

Supporting Portal 2: Community Edition (P2:CE) with this overlay is unfortunately not feasible. This document outlines the primary technical reasons.

### 1. Rendering Backend Mismatch
This overlay renders its UI via DirectX 9. P2:CE uses the Strata engine, which has been updated to use DirectX 11. This would require a complete rewrite of the rendering backend.

### 2. Engine API and FFI Instability
P2:CE's engine API is significantly different from Portal 2's, and it has removed the `plugin_load` command that this overlay relies on. Interfacing with the game would require substantial and difficult reverse-engineering.

Furthermore, the Strata source is closed and its API is subject to frequent, unannounced breaking changes. Any game update could break the overlay at any time, making stable support impossible to maintain.

### 3. The Recommended Path
The P2:CE developers themselves have suggested that it is better to ask them to implement desired features directly into the engine in C++.

Given these challenges, this project will remain focused on the official version of Portal 2 to ensure a stable and reliable experience.
