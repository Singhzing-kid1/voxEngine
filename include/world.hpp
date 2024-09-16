#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

class Shader;

class World{
    public:
        World(int, int);
        ~World();

        void init();
        void update();
        void render(mat4, mat4, mat4, Shader, vec3);

    private:
        struct Chunk{
            FloatGrid::Ptr voxelGrid;
            GLuint vao, vbo, ebo;
            vector<Vec3s> verticies;
            vector<unsigned int> indicies;
            vector<Vec3s> normals;
            Coord chunkPos;
            bool needsUpdate, newChunk;
        };
        
        struct worldCell {
            Coord coord;
            int height;
            worldCell(Coord _coord, int _height) : coord(_coord), height(_height) {}
        };

        int chunkSize;
        int worldHeight;

        vector<Chunk> chunks;

        bool isSurfaceVoxel(Coord, FloatGrid::Ptr);
        Vec3s voxelToWorld(Coord, Coord, float, float);

        Chunk generateChunkMesh(Chunk);
        Chunk updateChunk(Chunk);
        Chunk setupMeshBuffers(Chunk);
};

#endif