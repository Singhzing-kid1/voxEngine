#include "main.hpp"

Camera::Camera(float fov, float width, float height, float nearPlane, float farPlane, vec3 pos){
    this->fov = fov;
    this->aspect = height/width;
    this->nearPlane = nearPlane;
    this->farPlane = farPlane;
    this->pos = pos;
    translationVec = pos;
    this->up = vec3(0.0f, 1.0f, 0.0f);
    this->front = vec3(0.0f, 0.0f, -1.0f);
}

float Camera::getItem(float item){
    switch((int)item){
        case 1:
            return fov;

        case 2:
            return aspect;

        case 3:
            return nearPlane;

        case 4:
            return farPlane;

        default:
            return fov;
    }
}

vec3 Camera::getItem(int item){
    switch(item){
        case 1:
            return pos;

        case 2:
            return up;

        case 3:
            return front;

        case 4:
            return right;

        case 5:
            return translationVec;

        default:
            return pos;
    }
}

void Camera::setAngles(float yaw, float pitch){
    this->yaw = yaw;
    this->pitch = pitch;
}

void Camera::moveCamera(Camera::Axis axis, float power){
    switch(axis){
        case NORTH:
            translationVec += front * power * deltaTime;
            break;
        case SOUTH:
            translationVec -= front * power * deltaTime;
            break;
        case EAST:
            translationVec += right * power * deltaTime;
            break;
        case WEST:
            translationVec -= right * power * deltaTime;
            break;
        case NORTHEAST:
            translationVec += front * power * deltaTime;
            translationVec += right * power * deltaTime;
            break;
        case NORTHWEST:
            translationVec += front * power * deltaTime;
            translationVec -= right * power * deltaTime;
            break;
        case SOUTHEAST:
            translationVec -= front * power * deltaTime;
            translationVec += right * power * deltaTime;           
            break;
        case SOUTHWEST:
            translationVec-= front * power * deltaTime;
            translationVec -= right * power * deltaTime;
            break;
    } 
}

void Camera::update(float deltaTime){
    this->deltaTime = deltaTime;
    this->updateOrientation();
    this->calculateFront();
    this->calculateRight();
    this->calculatePos();
}

void Camera::updateOrientation(){
    float rYaw = radians(yaw);
    float rPitch = radians(pitch);

    quat qYaw = angleAxis(rYaw, vec3(0.0f, 1.0f, 0.0f));
    quat qPitch = angleAxis(rPitch, vec3(1.0f, 0.0f, 0.0f));

    cameraOrientation = normalize(qYaw * qPitch);
}

void Camera::calculateFront(){
    front = normalize(rotate(cameraOrientation, vec3(0.0f, 0.0f, -1.0f)));
}

void Camera::calculateRight(){
    right = normalize(cross(front, up));
}

void Camera::calculatePos(){
    pos = translationVec;
}