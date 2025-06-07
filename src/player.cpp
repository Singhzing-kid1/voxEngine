#include "main.hpp"

Player::Player(float fov, float height, float width, float nearPlane, float farPlane, vec3 pos, vec4 size, float speed) : Camera(fov, height, width, nearPlane, farPlane, pos){
    position = pos;
    playerTranslationVec = pos;
    this->size = size;
    this->speed = speed;
}

void Player::updatePlayer(float deltaTime, const Uint8* state, float yaw, float pitch, btCollisionWorld* collisionWorld){
    this->deltaTime = deltaTime;

    if(state[SDL_SCANCODE_W] && state[SDL_SCANCODE_D]){
        this->movePlayer(Camera::NORTHEAST, speed);
    } else if(state[SDL_SCANCODE_W] && state[SDL_SCANCODE_A]){
        this->movePlayer(Camera::NORTHWEST, speed);
    } else if(state[SDL_SCANCODE_S] && state[SDL_SCANCODE_D]){
        this->movePlayer(Camera::SOUTHEAST, speed);
    } else if(state[SDL_SCANCODE_S] && state[SDL_SCANCODE_A]){
        this->movePlayer(Camera::SOUTHWEST, speed);
    } else if(state[SDL_SCANCODE_W] && !state[SDL_SCANCODE_S]){
        this->movePlayer(Camera::NORTH, speed);
    } else if(state[SDL_SCANCODE_S] && !state[SDL_SCANCODE_W]){
        this->movePlayer(Camera::SOUTH, speed);
    } else if(state[SDL_SCANCODE_D] && !state[SDL_SCANCODE_A]){
        this->movePlayer(Camera::EAST, speed);
    } else if(state[SDL_SCANCODE_A] && !state[SDL_SCANCODE_D]){
        this->movePlayer(Camera::WEST, speed);
    }

    position = playerTranslationVec;
    translationVec = playerTranslationVec;
    this->setAngles(yaw, pitch);
    this->update(deltaTime);
}

void Player::movePlayer(Axis axis, float power){
    vec3 frontNoY = normalize(vec3(front.x, 0.0f, front.z));
    vec3 rightNoY = normalize(vec3(right.x, 0.0f, right.z));
    vec3 plannedMovement = vec3(0.0f);


    switch(axis){
        case NORTH:
            plannedMovement += frontNoY * power * deltaTime;
            break;
        case SOUTH:
            plannedMovement -= frontNoY * power * deltaTime; 
            break;
        case EAST:
            plannedMovement += rightNoY * power * deltaTime;
            break;
        case WEST:
            plannedMovement -= rightNoY * power * deltaTime;
            break;
        case NORTHEAST:
            plannedMovement += frontNoY * power * deltaTime;
            plannedMovement += rightNoY * power * deltaTime;
            break;
        case NORTHWEST:
            plannedMovement += frontNoY * power * deltaTime;
            plannedMovement -= rightNoY * power * deltaTime;
            break;
        case SOUTHEAST:
            plannedMovement -= frontNoY * power * deltaTime;
            plannedMovement += rightNoY * power * deltaTime;
            break;
        case SOUTHWEST:
            plannedMovement -= frontNoY * power * deltaTime;
            plannedMovement -= rightNoY * power * deltaTime;
            break;
    }

    playerTranslationVec += plannedMovement; 
}

