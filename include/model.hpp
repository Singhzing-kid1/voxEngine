#ifndef MODEL_HPP
#define MODEL_HPP

#include "main.hpp"

class Model{
    public:
        Model(const char*, int);

        vector<vec3> getPositions();
        vector<vec3> getColors();

        unsigned int getVao();

    
    private:

    vector<vec3> position;
    vector<vec3> color;

    unsigned int ebo, vbo, vao;

    float* vertices;
    unsigned int* indices;


};

#endif