# Vox(el) Engine

# why a custom engine?

tl;dr: over-ambitious and kinda dumb + otc game engines cant do what i want for same effort.

well i wanted to create a voxel based rogue-lite game, and i wanted to use  
proper voxel rendering techniques. initially i used traditional rendering techniques to achieve this,  
but since implementing vulkan, i have moved to using a raymarcher to render my voxels. initially my  
reasoning behind starting this project was that i wanted to learn the ins and outs of "over-the-counter"  
game engines like unity or unreal, but at this stage it might genuinely be more difficult to implement  
the rendering style i want in a game engine like unity.

## binaries

binaries for windows and linux are under [releases](https://github.com/Singhzing-kid1/voxEngine/releases/tag/v0.2.0-preAlpha)

## build

make sure you have the latest version of rust installed

clone the project and use `cargo build --release` to build.

the build will output @ `./target/release/`

## dev logs

check out [voxEngine devlogs](https://youtube.com/playlist?list=PLkGaFuDyjwONmwLIEOyiIF43p4a-F6Qlz&si=TiN8CTq6OB9KWOlf)

i also do devstreams occasionally on [twitch](https://twitch.tv/veersinghlive)

test correct branch?