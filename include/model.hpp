/**
 * @file model.hpp
 * @author Veer Singh
 * @brief Class definition to be able to render, transfrom, translate, and manage models
 * @version 0.0.3
 * @date 2024-07-27
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#ifndef MODEL_HPP
#define MODEL_HPP

#include "main.hpp"

class Shader;

/** 
 * @param modelPath Path of model file saved to disk
 * @param size Size of the model. Allows for scaling of a model.
 * @param chunkSpaceTransform Origin point of the model inside a chunk.
 * @param chunk XYZ coord of the chunk the model is in. 
 * @param shouldScalePositionBasedOnScaleOfVoxel determines wether or not the the spacing between voxels is scaled by the size of the voxel.
*/
class Model{
    public:
        Model(const char*, float, vec3, vec3, bool);

        vector<vec3> getPositions();
        vector<vec3> getColors();

        vec3 getChunkCoord();

        bool getShouldScalePositionBool();

        unsigned int getVao();


        unsigned int ebo, vbo, vao;

        void render(Shader, mat4, mat4, mat4);

    
    private:

    vector<vec3> position;
    vector<vec3> color;
    vec3 chunkSpaceTransform, chunk;
    float size;
    bool shouldScalePosition;

    float* vertices;
    unsigned int* indices;


};

#endif