# TODO

## Camera projection improvements 

* Decouple projection matrix from 3d Camera
* https://discordapp.com/channels/691052431525675048/743663924229963868/747328744653324288
* Change 3d camera to accept a Mat4 projection matrix, and provide a builder pattern for easy construction, instead of tying the camera3d to the PerspectiveProjection type.

## Cursor Picking

### Ray Cast Option 1: CPU - Use a system on Handle<Mesh> and Transform

https://discord.com/channels/691052431525675048/742884593551802431/747295381477523476

* Take the cursor position
* For each entity with a mesh and transform,
  * Take the mesh, and for each triangle
    * for each vertex
      * Take the position
      * Convert to a vec4
      * Multiply by the transform matrix to get it in the correct location
    * Store each vertex location for the triangle

### Ray Cast Option 2: GPU - picking shader

* Render each mesh with a different color to a buffer
* Use the cursor coordinates to get the color at that position in the buffer 
* Select the mesh with a lookup of the color using a hash table

O(n) render O(1) lookup
 This would not allow "select other" functionality (in z-depth) and may have
transparency problems (less so in CAD?), however this would be really good for hover highlight
and initial selection for responsivenes, the CPU method could be kicked off async after right
click to be ready if the suer clicks on "select other".


## Camera Manipulation

### Rotation center

Cast ray from mouse coordinate to first triangle intersection.

### Orbit cameras

#### Constrained Orbit

* Rotation center seems to stay fixed for constrained rotation?
* mouse_y = quat pitch
* mouse_x = quat yaw

#### Free Orbit
Axis of rotation: through rotation point, perpendicular to mouse vector

