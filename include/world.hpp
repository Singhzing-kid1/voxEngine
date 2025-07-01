#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

#define CHUNK_SIZE 32
#define VOXEL_SIZE 1

class Shader;

struct vecHash{
    size_t operator()(const vec3& v) const {
        size_t h = 0;
        h ^= hash<float>()(v.x) + 0x9e3779b9 + (h << 6) + (h >> 2);
        h ^= hash<float>()(v.y) + 0x9e3779b9 + (h << 6) + (h >> 2);
        h ^= hash<float>()(v.z) + 0x9e3779b9 + (h << 6) + (h >> 2);

        return h;
    }
};

struct worldGenInfo{
    PerlinNoise heightMap;
    PerlinNoise tempMap;
    PerlinNoise moistureMap;
};

struct Mesh{
    vector<vec3> verts, normals, colors;
    vector<unsigned int> inds;
};

struct Voxel{
    Voxel(int value) : value(value) {}
    Voxel() {}

    int value = -1;

    vector<vec3> neighbors = {vec3(0, 0, 1),
                                vec3(0, 1, 0),
                                vec3(1, 0, 0),
                                vec3(0, 0, -1),
                                vec3(0, -1, 0),
                                vec3(-1, 0, 0)};
};

class Chunk{
    public:
        Chunk(worldGenInfo, vec3, int);

        static constexpr float halfVoxelSize = (float)VOXEL_SIZE * 0.5f;

        void genMesh();
        Mesh getMesh(int);

        vector<vector<vector<Voxel>>> grid;

        vec3 origin;
        bool dirty = true;

        Mesh mesh;

    private:
        map<int, Mesh> lods;
        
        unordered_map<vec3, vector<vec3>, vecHash> faces {
            {vec3(0, 0, 1), {vec3(-1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1), vec3(1, -1, 1)}}, 
            {vec3(0, 1, 0), {vec3(-1, 1, -1), vec3(1, 1, 1), vec3(1, 1, -1), vec3(-1, 1, 1)}},
            {vec3(1, 0, 0), {vec3(1, -1, -1), vec3(1, 1, 1), vec3(1, -1, 1), vec3(1, 1, -1)}},
            {vec3(0, 0, -1), {vec3(-1, -1, -1), vec3(1, 1, -1), vec3(1, -1, -1), vec3(-1, 1, -1)}},
            {vec3(0, -1, 0), {vec3(-1, -1, -1),vec3(1, -1, 1), vec3(-1, -1, 1), vec3(1, -1, -1)}},
            {vec3(-1, 0, 0), {vec3(-1, -1, -1), vec3(-1, 1, 1), vec3(-1, 1, -1), vec3(-1, -1, 1)}}
        };

        bool checkOutOfBounds(vec3);
};

class World{
    public:
        World(float, int, int, int, String);

        void render(mat4, mat4, mat4, Shader, vec3);
        void update(vec3);

    private:
        vector<Chunk> world;
        worldGenInfo worldSeed;
        
        GLuint vao, vbo, ebo;
        size_t indsSize;
        
        bool buffered = false;

        int worldHeight, worldDimension, renderDist;

        void prepAndCombineBuffers(vec3);

        int hash(String);

};

#endif