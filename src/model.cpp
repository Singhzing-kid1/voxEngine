#include "main.hpp"

Model::Model(const char* modelPath, int size){
    ifstream model(modelPath);

    this->color.push_back(vec3(1, 0, 1));
    this->color.push_back(vec3(1, 0, 0));
    this->position.push_back(vec3(0, 0, 0));
    this->position.push_back(vec3(1, 1, 0));

    // ebo, vbo and vao start


    // ebo, vbo and vao end
}

unsigned int Model::getVao(){
    return this->vao;
}

vector<vec3> Model::getPositions(){
    return this->position;
}

vector<vec3> Model::getColors(){
    return this->color;
}