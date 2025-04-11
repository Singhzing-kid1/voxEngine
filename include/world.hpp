#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

class Shader;

class World{
    public:
        World(float, int, int, int);

        void update(vec3);
        void sync(vec3, int);
    
        void render(mat4, mat4, mat4, Shader, vec3);

    private:
        struct Region{
            FloatGrid::Ptr regionGrid;
            GLuint vao, vbo, ebo;
            vector<vec3> verts, normals, colors;
            vector<unsigned int> inds;
            Coord regionPos;
        };

        struct Mesh{
            bool isSurface;
            vector<Coord> directions;
        };

        map<Coord, vector<vec3>> faces {
            {Coord(0, 0, 1), {vec3(-1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1), vec3(1, -1, 1)}}, 
            {Coord(0, 1, 0), {vec3(-1, 1, -1), vec3(1, 1, 1), vec3(1, 1, -1), vec3(-1, 1, 1)}},
            {Coord(1, 0, 0), {vec3(1, -1, -1), vec3(1, 1, 1), vec3(1, -1, 1), vec3(1, 1, -1)}},
            {Coord(0, 0, -1), {vec3(-1, -1, -1), vec3(1, 1, -1), vec3(1, -1, -1), vec3(-1, 1, -1)}},
            {Coord(0, -1, 0), {vec3(-1, -1, -1),vec3(1, -1, 1), vec3(-1, -1, 1), vec3(1, -1, -1)}},
            {Coord(-1, 0, 0), {vec3(-1, -1, -1), vec3(-1, 1, 1), vec3(-1, 1, -1), vec3(-1, -1, 1)}}};
        
        FloatGrid::Ptr world;
        Region region;

        int worldHeight, renderDist; 
        float voxelSize;
        bool newWorld = true;

        Mesh isSurfaceVoxel(Coord, FloatGrid::Accessor);
        vec3 voxelToWorld(Coord, Coord, float);        

        void genRegionMesh();
        void updateRegion();
        void setupMeshBuffers();


};

#endif