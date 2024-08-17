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
    vec3 frontNoY = normalize(vec3(cameraFront.x, 0.0f, cameraFront.z));
    vec3 rightNoY = normalize(vec3(cameraRight.x, 0.0f, cameraRight.z));

    switch (axis) {
        case NORTH:
            this->transVec += frontNoY * scalar;
            break;
        case SOUTH:
            this->transVec -= frontNoY * scalar;
            break;
        case EAST:
            this->transVec += rightNoY * scalar;
            break;
        case WEST:
            this->transVec -= rightNoY * scalar;
            break;
        case NORTHEAST:
            this->transVec += frontNoY * scalar;
            this->transVec += rightNoY * scalar;
            break;
        case NORTHWEST:
            this->transVec += frontNoY * scalar;
            this->transVec -= rightNoY * scalar;
            break;
        case SOUTHEAST:
            this->transVec -= frontNoY * scalar;
            this->transVec += rightNoY * scalar;
            break;
        case SOUTHWEST:
            this->transVec -= frontNoY * scalar;
            this->transVec -= rightNoY * scalar;
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