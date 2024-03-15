#include "main.hpp"


// public
Camera::Camera(float fov, float height, float width, float nearPlane, float farPlane, vec3 pos){
    this->fov = radians(fov);
    this->aspect = height / width;
    this->nearPlane = nearPlane;
    this->farPlane = farPlane;
    this->cameraPos = pos;
    this->cameraUp = vec3(0.0f, 1.0f, 0.0f);
    this->cameraFront = vec3(0.0f, 0.0f, 1.0f);
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

void Camera::moveCamera(string axis, float scalar){

    if(axis == "ad"){
        this->transVec += this->cameraRight * scalar;
    }
    
    if (axis == "ws"){
        this->transVec += this->cameraFront * scalar;
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

void Camera::update(){   
    this->calculateRightVec();
    this->calculateFrontVec();
    this->calculatePosVec();
}

// private

void Camera::calculateFrontVec(){
    cameraFront.x = sin(radians(yaw)) * cos(radians(pitch));
    cameraFront.y = sin(radians(pitch));
    cameraFront.z = cos(radians(yaw)) * cos(radians(pitch));
 }

void Camera::calculatePosVec(){
    cameraPos = transVec;
}

void Camera::calculateRightVec(){
    cameraRight.x = sin(radians(yaw) - 3.14f/2.0f);
    cameraRight.y = 0.0f;
    cameraRight.z = cos(radians(yaw) - 3.14f/2.0f);
}