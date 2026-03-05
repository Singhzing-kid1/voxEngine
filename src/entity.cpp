#include "main.hpp"

Entity::Entity(float mass, vec3 size, vec3 position) : 
    mass(mass), 
    size(size),
    position(position),
    netForce(vec3(0)) {}

void Entity::addForce(vec3 force){
    netForce += force;
}

void Entity::calculateAcceleration(){
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
}
