#include "main.hpp"


// public
Camera::Camera(float fov, float height, float width, float nearPlane, float farPlane, vec3 pos){
    this->fov = radians(fov);
    this->aspect = height / width;
    this->nearPlane = nearPlane;
    this->farPlane = farPlane;
    this->cameraPos = pos;
    this->cameraUp = vec3(0.0f, 1.0f, 0.0f);
    this->cameraFront = vec3(0.0f, 0.0f, -1.0f);
}

vec3 Camera::getFrontVec(){
    return this->cameraFront;
}

vec3 Camera::getPosVec(){
    return this->cameraPos;
}

vec3 Camera::getUpVec(){
    return this->cameraUp;
}

void Camera::moveCamera(Axis axis, float scalar){
    switch(axis){
        case NORTH:
            this->transVec += this->cameraFront * scalar;
            break;
        case SOUTH:
            this->transVec -= this->cameraFront * scalar;
            break;
        case EAST:
            this->transVec += this->cameraRight * scalar;
            break;
        case WEST:
            this->transVec -= this->cameraRight * scalar;
            break;
        case NORTHEAST:
            this->transVec += this->cameraFront * scalar;
            this->transVec += this->cameraRight * scalar;
            break;
        case NORTHWEST:
            this->transVec += this->cameraFront * scalar;
            this->transVec -= this->cameraRight * scalar;
            break;
        case SOUTHEAST:
            this->transVec -= this->cameraFront * scalar;
            this->transVec += this->cameraRight * scalar;           
            break;
        case SOUTHWEST:
            this->transVec -= this->cameraFront * scalar;
            this->transVec -= this->cameraRight * scalar;
            break;
    }       
}

float Camera::getFov(){
    return this->fov;
}

float Camera::getAspect(){
    return this->aspect;
}

float Camera::getNear(){
    return this->nearPlane;
}

float Camera::getFar(){
    return this->farPlane;
}

void Camera::setPitch(float pitch){
    this->pitch = pitch;
}

void Camera::setYaw(float yaw){
    this->yaw = yaw;
}

void Camera::setPos(vec3 position){
    this->transVec = position;
}

void Camera::update(){
    this->updateOrientation();
    this->calculateRightVec();
    this->calculateFrontVec();
    this->calculatePosVec();
}

// private

void Camera::updateOrientation(){
    float rYaw = radians(yaw);
    float rPitch = radians(pitch);

    quat qYaw = angleAxis(rYaw, vec3(0.0f, 1.0f, 0.0f));
    quat qPitch = angleAxis(rPitch, vec3(1.0f, 0.0f, 0.0f));

    cameraOrientation = normalize(qYaw * qPitch);
}


void Camera::calculateFrontVec(){
    cameraFront = normalize(rotate(cameraOrientation, vec3(0.0f, 0.0f, -1.0f)));
 }

void Camera::calculatePosVec(){
    cameraPos = transVec;
}

void Camera::calculateRightVec(){
    cameraRight = normalize(cross(cameraFront, cameraUp));
}