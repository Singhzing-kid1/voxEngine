#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

#define CHUNK_SIZE 32

class Shader;

template <typename T> class Coord{
    public:
        Coord(T x, T y, T z) : x(x), y(y), z(z) {}
        template <typename U> Coord(const Coord<U>& other) : x(static_cast<T>(other.x)), y(static_cast<T>(other.y)), z(static_cast<T>(other.z)) {}
        Coord();

        Coord& operator=(const Coord&);
        bool operator==(const Coord&);
        bool operator>=(const Coord&);
        bool operator<=(const Coord&);
        bool operator!=(const Coord&);
        bool operator<(const Coord&) const;
        template <typename U> friend ostream& operator<<(ostream&, const Coord<U>&);

        Coord offset(Coord);

        operator vec3() const; 

        T x, y, z;
};

#define fCoord Coord<float>
#define iCoord Coord<int>

struct worldGenInfo{
    PerlinNoise heightMap;
    PerlinNoise tempMap;
    PerlinNoise moistureMap;
};

struct Mesh{
    vector<vec3> verts, normals, colors;
    vector<unsigned int> inds;
};

class Voxel{
    public:
        Voxel(fCoord, int, int, PerlinNoise);
        Voxel();

        fCoord position;
        int value;

        vector<iCoord> faces;
        bool isSurface;

    private:
        void determineFaces(PerlinNoise, int);
        vector<fCoord> directions = {fCoord(0, 0, 1),
                                     fCoord(0, 1, 0),
                                     fCoord(1, 0, 0),
                                     fCoord(0, 0, -1),
                                     fCoord(0, -1, 0),
                                     fCoord(-1, 0, 0) };
}; 

class Chunk{
    public:
        Chunk(worldGenInfo, vec3, int);

        void genMesh();
        Mesh getMesh(int);

        Voxel grid[CHUNK_SIZE][CHUNK_SIZE][CHUNK_SIZE];

        vec3 origin;
        bool dirty = true;
        bool buffered = false;

    private:
        map<int, Mesh> lods;
        
        map<iCoord, vector<vec3>> faces {
            {iCoord(0, 0, 1), {vec3(-1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1), vec3(1, -1, 1)}}, 
            {iCoord(0, 1, 0), {vec3(-1, 1, -1), vec3(1, 1, 1), vec3(1, 1, -1), vec3(-1, 1, 1)}},
            {iCoord(1, 0, 0), {vec3(1, -1, -1), vec3(1, 1, 1), vec3(1, -1, 1), vec3(1, 1, -1)}},
            {iCoord(0, 0, -1), {vec3(-1, -1, -1), vec3(1, 1, -1), vec3(1, -1, -1), vec3(-1, 1, -1)}},
            {iCoord(0, -1, 0), {vec3(-1, -1, -1),vec3(1, -1, 1), vec3(-1, -1, 1), vec3(1, -1, -1)}},
            {iCoord(-1, 0, 0), {vec3(-1, -1, -1), vec3(-1, 1, 1), vec3(-1, 1, -1), vec3(-1, -1, 1)}}
        };
};

class World{
    public:
        World(float, int, int, int, String);

        void render(mat4, mat4, mat4, Shader, vec3);
        void update();

    private:
        vector<Chunk> world;
        worldGenInfo worldSeed;
        
        GLuint vao, vbo, ebo;
        size_t indsSize;

        int worldHeight, worldDimension, renderDist;

        void prepAndCombineBuffers();

        int hash(String);

};

#endif