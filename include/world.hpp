#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

#define VOXEL_SIZE 0.1
#define CHUNK_SIZE 32
#define MAX_WORLD_GENERATION_HEIGHT 64

class Shader;

struct vec3Hash{
    size_t operator()(const vec3& v) const {
        size_t h = 0;
        h ^= hash<float>()(v.x) + 0x9e3779b9 + (h << 6) + (h >> 2);
        h ^= hash<float>()(v.y) + 0x9e3779b9 + (h << 6) + (h >> 2);
        h ^= hash<float>()(v.z) + 0x9e3779b9 + (h << 6) + (h >> 2);

        return h;
    }
};

struct vec2Hash{
    size_t operator()(const vec2& v) const {
        size_t h = 0;
        h ^= hash<float>()(v.x) + 0x9e3779b9 + (h << 6) + (h >> 2);
        h ^= hash<float>()(v.y) + 0x9e3779b9 + (h << 6) + (h >> 2);

        return h;
    }
};

struct noiseMaps{
    PerlinNoise heightMap;
    PerlinNoise moistureMap;
    PerlinNoise tempMap;
};

template <typename T>
struct Box {
    Box(T min, T max) : min(min), max(max) {}
    Box() {}

    T min, max;

    bool isInside(T coord){
        return all(greaterThanEqual(coord, min)) && all(lessThanEqual(coord, max));
    }
};

#define Box3 Box<vec3>
#define Box2 Box<vec2>

struct Mesh{
    vector<vec3> vertices, colors, normals;
    vector<unsigned int> indices;
};

class Chunk{ 
    public:
        Chunk(noiseMaps, vec2, int);
        ~Chunk();

        enum class VOXEL {
            EMPTY = -2,
            AIR = -1,
            HOTDRY = 1, 
            HOTNORM = 2, 
            HOTWET = 3, 
            NORMDRY = 4, 
            NORM = 5, 
            NORMWET = 6, 
            COLDDRY = 7, 
            COLDNORM = 8, 
            COLDWET = 9
        };

        void genMesh(unordered_map<vec2, Chunk, vec2Hash>&);

        vector<vector<vector<VOXEL>>> grid;

        bool buffered = false;
        bool genBuffers = false;
        bool loaded = false;
        bool dirty = false;
        bool needsToBeUnloaded = false;

        GLuint vao, vbo, ebo;

        vec2 origin;
        Mesh mesh;

    private:
        static constexpr float halfVoxelSize = (float)VOXEL_SIZE * 0.5f;
        
        // should be a .json or similar just for testing
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

        noiseMaps maps;
        int worldHeight;

        static const unordered_map<vec3, vector<vec3>, vec3Hash> faces;
        static const map<VOXEL, vec3> blockType;
        static const vector<vec3> neighbors;

        bool outOfBounds(vec3);
        bool layerEmpty(int);
        bool rowEmpty(int, int);
};

class World {
    public:
        World(int, int, int, std::string, vec2);
        ~World();

        enum class REQUEST {
            CHUNKCREATION,
            GENMESH,
            LOAD,
            UPDATE,
            UNLOAD
        };

        void update(vec3, float);
        void sendBuffers();
        void cleanUpRenderable();

        void render(mat4, mat4, mat4, vec3, Shader);

    private:
        unordered_map<vec2, Chunk, vec2Hash> world;
        int worldHeight, worldDimension, renderDist;

        vector<Chunk> renderable;

        thread reqThread;
        condition_variable requestQueueCV;

        mutex worldMutex;
        mutex queueMutex;
        mutex renderableMutex;

        atomic<bool> running{true};
        atomic<bool> working;
        bool shouldRun = true;
        bool edited;

        static constexpr int requestsPerFrame = 5;

        queue<pair<REQUEST, vec2>> requestQueue;
        Box2 renderBox;
        noiseMaps maps;

        vec2 lastPlayerPos;

        void requestManager();

        unsigned int hash(std::string);
};
#endif