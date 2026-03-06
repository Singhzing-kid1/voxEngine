#ifndef ENTITY_HPP
#define ENTITY_HPP

#include "main.hpp"

class Entity{
    public:
        Entity(float, vec3, vec3);

        void addAppliedForce(vec3);
        void addNormalForce(vec3);

        float mass;

        vec3 size;
        vec3 position;
        vec3 velocity;
        vec3 netForce;
        vec3 normalForce;
        vec3 appliedForce;

        float rYaw;
        float rPitch;

    private:
        vec3 acceleration;

        void calculateAcceleration();
        void calculateVelocity(float);
        void calculatePosition(float);

    protected:
        void tick(float);

};

#endif