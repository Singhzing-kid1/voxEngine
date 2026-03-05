#ifndef PHYSICS_HPP
#define PHYSICS_HPP

#include "main.hpp"

class World;
class Entity;
class Shader;

class Physics {
    public:
        Physics(float);

        template <typename T>
        void addEntity(T& entity){
            static_assert(is_base_of_v<Entity, T>, "Must derive from Entity");
            entities.emplace_back(ref(entity));
        }

        void step(bool);

        btCollisionWorld* collisionWorld;
        
    private:
        vector<reference_wrapper<Entity>> entities;

        float gravity;
        vec3 gravityDirection = vec3(0.0f, -1.0f, 0.0f);

        vector<vec3> directions = {vec3(0.0f, 0.0f, 1.0f),
                                   vec3(0.0f, 1.0f, 0.0f),
                                   vec3(1.0f, 0.0f, 0.0f),
                                   vec3(0.0f, 0.0f, -1.0f),
                                   vec3(0.0f, -1.0f, 0.0f),
                                   vec3(-1.0f, 0.0f, 0.0f)};


        bool collisionCheck(vec3, vec3, vec3, pair<float, float>);
};


#endif