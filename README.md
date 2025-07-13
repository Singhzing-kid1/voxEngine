# Vox Engine

NOTE:
i started redoing this project between v0.0.5 and v0.0.6 to have better mass rendering optimizations.
v0.0.6 has been REDONE completely from scratch. v0.0.7 will also be redone from scratch using the stuff i learned making v0.0.6, but with vulkan so i can use modern graphics techniques


i messed up guys  
vulkan  
such a pain  
not doing that  
going back to openGL  

![ver sad](https://external-content.duckduckgo.com/iu/?u=https%3A%2F%2Fi.ytimg.com%2Fvi%2FpLe-hanLTPs%2Fmaxresdefault.jpg&f=1&nofb=1&ipt=7bd33c630750f30d20282f5ac1556747157742d5f72b7a086686d43e542af2a3&ipo=images "sad hampter")

0.0.7 will be openGL w/ openDB for voxel storage and bullet physics for collisions :) <--- might use rigid bodies ;) well see.

Gonna try to get this (0.0.7) out for end of March.

0.0.7 almost done. trying to sort out so e optimization stuff with parallelization of syncing the world and meshing the world. might also implement a version of a greedy mesher.



## For you summer of making people

i have used AI on this project, mostly to find information, debug, and occasionally write some code :). feel free to download any of the previous versions for yourself and check it out <3



update func()
check to see if playerPos has changed by at least 3 chunks from last frame OR edit true
    update renderBox
    iter through renderBox
        if E 
            if dirty
                add gen mesh request
                set buffered false
            if !dirty
                cont
        if !E 
            add chunk creation request
            set buffered false


requestManager(threaded) func
if requestQueue
    loop to max requestPerFrame
        take one from queue
        if genMeshRequest
            req.chunkCoord
            get chunk from world by ref
            call genMesh
            set chunk.dirty to false
        if chunkCreationRequest
            create chunk using req.chunkCoord
            call genMesh
            set chunk.dirty to false
            add to worldHashMap
            add to renderable vector

buffer(threaded) func
    if buffered
        return
    loop through renderable vector by ref
        add ch.verts, ch.inds, ch.norms, ch.colors to temp vectors as such
    set indsize sizeof inds(temp)
    set opengl buffers




            
