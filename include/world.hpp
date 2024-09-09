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
        void render(mat4, mat4, mat4, Shader);

    private:
        struct Chunk{
            FloatGrid::Ptr voxelGrid;
            GLuint vao, vbo, ebo;
            vector<Vec3s> verticies;
            vector<unsigned int> indicies;
            vector<Vec3s> normals;
            bool needsUpdate, newChunk;
        };

        int chunkSize;
        int worldHeight;

        vector<Chunk> chunks;

        Chunk generateChunkMesh(Chunk);
        Chunk updateChunk(Chunk);
        Chunk setupMeshBuffers(Chunk);
};

#endif