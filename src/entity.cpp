#include "main.hpp"

Entity::Entity(float mass, vec3 size, vec3 position) : 
    mass(mass), 
    size(size),
    position(position),
    netForce(vec3(0)) {}

void Entity::addAppliedForce(vec3 force){
    appliedForce += force;
}

void Entity::addNormalForce(vec3 force){
    normalForce += force;
}

void Entity::calculateAcceleration(){
    netForce += appliedForce + normalForce;
    acceleration = netForce/mass;
}

void Entity::calculateVelocity(float deltaTime){
    velocity += acceleration * deltaTime;
}

void Entity::calculatePosition(float deltaTime){
    position += velocity * deltaTime;
}

void Entity::tick(float deltaTime){
    calculateAcceleration();
    calculateVelocity(deltaTime);
    calculatePosition(deltaTime);
    netForce = vec3(0);
    appliedForce = vec3(0);
    normalForce = vec3(0);
}
