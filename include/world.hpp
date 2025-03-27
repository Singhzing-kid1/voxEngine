#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

class Shader;

class World{
    public:
        World(float, int, int, int);

        void update(vec3, int);
        // check time
        // resync region every 5 seconds
        // update region vao, vbo, ebo
    
        void render(mat4, mat4, mat4, Shader, vec3);

    private:
        struct Region{
            FloatGrid::Ptr regionGrid;
            GLuint vao, vbo, ebo;
            vector<vec3> verts, normals;
            vector<unsigned int> inds;
            Coord regionPos;
            bool updateRegion, updateWorld;
        };
        
        FloatGrid::Ptr world;
        Region region;

        int worldHeight, renderDist; 
        float voxelSize;
        bool newWorld = true;

        bool isSurfaceVoxel(Coord, FloatGrid::Ptr);
        vec3 voxelToWorld(Coord, Coord, float);

        void sync(vec3);

        void genRegionMesh();
        void updateRegion();
        void setupMeshBuffers();


};

#endif