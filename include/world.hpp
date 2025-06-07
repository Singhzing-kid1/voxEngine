#ifndef WORLD_HPP
#define WORLD_HPP

#include "main.hpp"

class Shader;

class World{
    public:
        World(float, int, int, int, std::string);

        enum MouseButton {LMB, RMB};

        void update();
        void sync(vec3);
        void manipulate(Coord, MouseButton);
    
        void render(mat4, mat4, mat4, Shader, vec3);
        
    private:
        GLuint vao, vbo, ebo;
        vector<vec3> verts, normals, colors;
        vector<unsigned int> inds;

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
            {Coord(-1, 0, 0), {vec3(-1, -1, -1), vec3(-1, 1, 1), vec3(-1, 1, -1), vec3(-1, -1, 1)}}
        };
        

        map<float, vec3> blockType {
            {1.0f, vec3(1, 0, 0)},
            {2.0f, vec3(1, 0.3, 0)},
            {3.0f, vec3(0.8, 0.29, 0.16)},
            {4.0f, vec3(1, 0.65, 0)},
            {5.0f, vec3(0.93, 0.76, 0)},
            {6.0f, vec3(0.76, 0.9, 0.027)},
            {7.0f, vec3(0.18, 0.87, 0.27)},
            {8.0f, vec3(0.15, 0.95, 0.45)},
            {9.0f, vec3(0.062, 0.97, 0.85)}
        };

        FloatGrid::Ptr world;

        int worldHeight, renderDist; 
        float voxelSize;
        bool newWorld = true;

        Mesh isSurfaceVoxel(Coord, FloatGrid::Accessor);
        vec3 voxelToWorld(Coord, float);        

        void genRegionMesh();
        void genBulletHitbox(btCollisionWorld*);
        void setupMeshBuffers();

        int hash(std::string);



};

#endif