#include "main.hpp"

Player::Player(float fov, float height, float width, float nearPlane, float farPlane, vec3 pos, vec3 size) : Camera(fov, height, width, nearPlane, farPlane, pos) {
    this->position = pos;
    this->size = size;
};

// public: 

void Player::pUpdate(vector<Model> models, btCollisionWorld* collisionWorld){
    this->position = this->cameraPos;
    this->velocity = (this->position - this->prevPos) / this->deltaTime;
    this->prevPos = this->position;
    this->checkForCollisions(models, collisionWorld);
    this->update(); 

}

vec3 Player::getPlayerPos(){
    return this->position;
}

vec3 Player::getPlayerVelocity(){
    return this->velocity;
}

void Player::setDeltaTime(float deltaTime){
    this->deltaTime = deltaTime;
}

void Player::movePlayer(Axis axis, float scalar, btCollisionWorld* collisionWorld){
    vec3 frontNoY = normalize(vec3(this->cameraFront.x, 0.0f, this->cameraFront.z));
    vec3 rightNoY = normalize(vec3(this->cameraRight.x, 0.0f, this->cameraRight.z));
    vec3 plannedMovement = vec3(0.0f);

    switch (axis) {
        case NORTH:
            plannedMovement += this->blockedForward ? vec3(0) : frontNoY * scalar;
            break;
        case SOUTH:
            plannedMovement -= this->blockedBackward ? vec3(0) : frontNoY * scalar;
            break;
        case EAST:
            plannedMovement += this->blockedRight ? vec3(0) : rightNoY * scalar;
            break;
        case WEST:
            plannedMovement -= this->blockedLeft ? vec3(0) : rightNoY * scalar;
            break;
        case NORTHEAST:
            plannedMovement += this->blockedForward ? vec3(0) : frontNoY * scalar;
            plannedMovement += this->blockedRight ? vec3(0) : rightNoY * scalar;
            break;
        case NORTHWEST:
            plannedMovement += this->blockedForward ? vec3(0) : frontNoY * scalar;
            plannedMovement -= this->blockedLeft ? vec3(0) : rightNoY * scalar;
            break;
        case SOUTHEAST:
            plannedMovement -= this->blockedBackward ? vec3(0) : frontNoY * scalar;
            plannedMovement += this->blockedRight ? vec3(0) : rightNoY * scalar;
            break;
        case SOUTHWEST:
            plannedMovement -= this->blockedBackward ? vec3(0) : frontNoY * scalar;
            plannedMovement -= this->blockedLeft ? vec3(0) : rightNoY * scalar;
            break;
    }

    this->transVec += plannedMovement;

}


// private:

void Player::checkForCollisions(vector<Model> models, btCollisionWorld* collisionWorld){
    this->hitObject = false;

    for(auto m : models){
        if(m.getCanCollide()){
            for(auto v : m.getVoxelPointsInGlobal()){
                bool x = abs(this->position.x - v.x) <= 1.0 * (m.getSize() + (0.5 * this->size.x)) ? true : false; 
                bool y = abs(this->position.y - v.y) <= 1.0 * (m.getSize() + (0.5 * this->size.y)) ? true : false;
                bool z = abs(this->position.z - v.z) <= 1.0 * (m.getSize() + (0.5 * this->size.z)) ? true : false;

                if(x && y && z){
                    this->hitObject = true;
                    break;
                }
            }
            if(this->hitObject){
                break;
            }

        }
    }

    bool* blockedDirections[6] = {&blockedRight, &blockedUp, &blockedForward, &blockedLeft, &blockedDown, &blockedBackward};

    if(this->hitObject){
        for(int x = 0; x < 6; x++){
            float rYaw = radians(this->yaw);
            quat qYaw = angleAxis(rYaw, vec3(0.0f, 1.0f, 0.0f));

            vec3 rotatedDirection = normalize(rotate(qYaw, this->directions[x]));

            btVector3 rayStart(this->position.x, this->position.y, this->position.z);
            btVector3 rayEnd(this->position.x + rotatedDirection.x, this->position.y + rotatedDirection.y, this->position.z + rotatedDirection.z);

            btCollisionWorld::ClosestRayResultCallback rayCallback(rayStart, rayEnd);
            collisionWorld->rayTest(rayStart, rayEnd, rayCallback);

            *blockedDirections[x] = rayCallback.hasHit();
        }
    } else {
        this->blockedUp = false;
        this->blockedDown = false;
        this->blockedLeft = false;
        this->blockedRight = false;
        this->blockedForward = false;
        this->blockedBackward = false;
    }
}