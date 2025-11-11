Actively developing and changing. 


# DOING
* Verify grid steadily updates thumbnails without UI blanking
* The Grid View is WIP. presently the second render layer is taking over the full view.
* The scene camera is also going full window...

# TODO
* Auto-frame models for thumbnails
* Only show placeholder until thumbnail Ready state
* Fix File Dialog Popup
* File name in top panel
  * Rename files
  * Move Files
  * Delete Files
* Favorite Folders
* Zoom Extents of model
* Info about model in right panel
* Shadows toggle
* Arrow keys for next/previous in list
* Sort list by name or date


# DONE
* Test GLTF thumbnail rendering in grid view
* Queue-based one-at-a-time thumbnail generation
* Dedicated single thumbnail RenderLayer (7) and offscreen camera rendering to Image
* Restrict main world camera/light to layer 0 and raise draw order; prevent GLTF cameras from taking over the window; UI stays visible
* Disable GLTF cameras in loaded scenes during thumbnail rendering
* Apply thumbnail render layer to entire GLTF scene hierarchy
* Defer thumbnail capture until assets load (avoid grey clears)
* Scope cleanup to per-file entities and fix despawn warnings
* Implement GLTF thumbnail rendering system
* Create thumbnail cache with render-to-texture
* Integrate thumbnails into file grid
* Test and verify toggle functionality
* Button styling
* Sorting
* Integrate toggle logic into ui_system
* Render grid of 2D cards in grid mode
* Add toggle button to UI
* Add state variable for view mode (3D/grid)
* Support GLTF
* Setup draggable panes
* Setup 3d view
* Pan orbit cam
* Basic styling for buttons
