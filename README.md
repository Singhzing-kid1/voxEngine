# Vox Engine

## Build

There are prebuilds

to compile from source:
dependencies are
- glew
- sdl2
- sdl2_ttf
- opengl
- glm
- bullet physics

the prebuilds include the required dlls

compile using cmake, there is an included cmakelists.txt

## other random stuff mostly for the friends(testers)

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


## some algorithm rambling to help me think through things(this is very much a hobby project)

update func()
if playerPos changed by 3 chunks within 30 seconds, any chunk has been edited, or loading spawn chunks
update renderBox if needed
iter through renderBox
    if E
        if edited 
            send gen mesh request
        if !loaded
            send load chunk request
    if !E
        send chunk creation request

iter through all chunks:
    skip chunks in renderBox
    send unload request


request Manager func(threaded)

iter through requests(max 5 per frame)
    if request is chunkcreation:
        create chunk
        insert into world hashmap
        send gen_mesh request

    if request is gen_mesh:
        generate mesh
        set dirty flag false
        check loaded flag:
            if loaded:
                send update request
            if !loaded:
                send load request

    if request is load:
        set loaded flag to true
        copy chunk to renderable array

    if request is update:
        find chunk in renderable array
        reinsert chunk in renderable array
    
    if request is unload:
        find chunk in renderable array
        find chunk in world hashmap
        set flag loaded to false
        delete chunk from renderable array

clickup integration test

        
        


            
