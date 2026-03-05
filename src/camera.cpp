#include "main.hpp"

Camera::Camera(float fov, float width, float height, float nearPlane, float farPlane, vec3 pos) : 
    fov(fov), 
    aspect(height/width), 
    nearPlane(nearPlane), 
    farPlane(farPlane), 
    cameraPosition(pos), 
    up(vec3(0.0f, 1.0f, 0.0f)), 
    front(vec3(0.0f, 0.0f, -1.0f)) {}

void Camera::setAngles(float yaw, float pitch){
    this->yaw = yaw;
    this->pitch = pitch;
}

void Camera::update(float deltaTime){
    deltaTime = deltaTime;
    updateOrientation();
    calculateRight();
    calculateFront();
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