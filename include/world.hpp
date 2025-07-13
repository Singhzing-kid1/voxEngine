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

    Box() {}

    vec3 min, max;
    bool isInside(vec3 position){
        return all(lessThanEqual(min, position)) || all(greaterThanEqual(position, max));
    }

};

class Chunk{
    public:
        Chunk(worldGenInfo, vec3, int);

        static constexpr float halfVoxelSize = (float)VOXEL_SIZE * 0.5f;
        
        // TODO, put these values in a yml or similar

        static constexpr float tempColdMin = -1.0f;
        static constexpr float tempColdMax = -0.3f;
        static constexpr float tempNormMin = -0.3f;
        static constexpr float tempNormMax = 0.3f;
        static constexpr float tempHotMin = 0.3f;
        static constexpr float tempHotMax = 1.0f;

        static constexpr float dryMin = 0.0f;
        static constexpr float dryMax = 0.4f;
        static constexpr float normMin = 0.4f;
        static constexpr float normMax = 0.7f;
        static constexpr float wetMin = 0.7f;
        static constexpr float wetMax = 1.0f;

        static constexpr float heightMapScalar = 0.01f;
        static constexpr float tempAndMoistureMapScalar = 0.005f;

        static constexpr int perlinNoiseOctaveAmount = 4;
        

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

        void genMesh(const unordered_map<vec3, Chunk, vecHash>&);
        Mesh getMesh(int);

        vector<VOXEL> grid;

        vec3 origin;
        bool dirty = true;

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
        World(float, int, int, int, String, vec3);
        ~World();

        enum class REQUEST {CHUNKCREATE, GENERATEMESH};

        void render(mat4, mat4, mat4, Shader, vec3);

        void update(vec3);
        void requestManager();
        void prepAndCombineMeshes();
        void setBuffers();

        mutex worldMutex;
        mutex renderableMutex;
        mutex requestQueueMutex;
        mutex indsSizeMutex;
        mutex meshMutex;
        condition_variable requestQueueCV;

        thread reqThread;
        thread bufThread;


        bool running = true;

    private:
        unordered_map<vec3, Chunk, vecHash> world;
        vector<Chunk*> renderable;
        Box renderBox;

        static constexpr int requestsPerFrame = 5;

        queue<pair<REQUEST, vec3>> requests;
        
        worldGenInfo worldSeed;

        GLuint vao, vbo, ebo;
        Mesh worldMesh;
        
        atomic<bool> buffered{false};
        bool edited = false;

        int worldHeight, worldDimension, renderDist;
        vec3 lastPlayerPos = vec3(0);

        int hash(String);

};

#endif