#include "main.hpp"

Player::Player(float fov, float height, float width, float nearPlane, float farPlane, vec3 pos, vec4 size, float movementForce, float mass) : 
    Entity(mass, size, pos),
    Camera(fov, height, width, nearPlane, farPlane, pos), 
    movementForce(movementForce) {}

variant<float, vec3> Player::getItem(Player::ITEM item){
    switch(item){
        case FOV:
            return fov;
        
        case ASPECT:
            return aspect;

        case NEAR:
            return nearPlane;

        case FAR:
            return farPlane;

        case POSITION:
            return position;

        case CAMERAPOSITION:
            return cameraPosition;

        case UP:
            return up;

        case FRONT:
            return front;

        case RIGHT:
            return right;
    }
}

void Player::updatePlayer(float deltaTime, const Uint8* state, float yaw, float pitch){
    this->deltaTime = deltaTime;

    vec3 frontHorizontal = normalize(vec3(front.x, 0.0f, front.z));
    vec3 rightHorizontal = normalize(vec3(right.x, 0.0f, right.z));

    if(state[SDL_SCANCODE_W]){
        addForce(movementForce * frontHorizontal);
    } 
    
    if(state[SDL_SCANCODE_S]){
        addForce(-movementForce * frontHorizontal);
    }

    if(state[SDL_SCANCODE_D]){
        addForce(movementForce * rightHorizontal);
    } 

    if(state[SDL_SCANCODE_A]){
        addForce(-movementForce * rightHorizontal);
    }

    cameraPosition = position;
    
    this->yaw = yaw;
    this->pitch = pitch;

    rYaw = radians(yaw);
    rPitch = radians(pitch);

    update(deltaTime);
    tick(deltaTime);
}

