#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

#define CHUNK_SIZE 32
#define VOXEL_SIZE 0.05

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

struct Box{
    Box(vec3 min, vec3 max) : min(min), max(max) {}

    vec3 min, max;
    bool isInside(vec3 position){
        return all(lessThanEqual(min, position)) || all(greaterThanEqual(position, max));
    }
};

class Chunk{
    public:
        Chunk(worldGenInfo, vec3, int);

        static constexpr float halfVoxelSize = (float)VOXEL_SIZE * 0.5f;

        enum class VOXEL {EMPTY = -1,
                          HOTDRY = 1, 
                          HOTNORM = 2, 
                          HOTWET = 3, 
                          NORMDRY = 4, 
                          NORM = 5, 
                          NORMWET = 6, 
                          COLDDRY = 7, 
                          COLDNORM = 8, 
                          COLDWET = 9};

        void genMesh(const vector<Chunk>&);
        Mesh getMesh(int);

        vector<VOXEL> grid;

        vec3 origin;
        bool dirty = true;
        bool buffered = false;

        Mesh mesh;

    private:
        map<int, Mesh> lods;

        worldGenInfo genInfo;
        int worldHeight;
        
        static const unordered_map<vec3, vector<vec3>, vecHash> faces; 

        static const map<VOXEL, vec3> blockType;

        static const vector<vec3> neighbors; 

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