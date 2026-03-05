#ifndef CAMERA_HPP
#define CAMERA_HPP

#include "main.hpp"

class Camera{
    public:
        Camera(float, float, float, float, float, vec3);
        
        void update(float);

        void setAngles(float, float);

    private:

        quat cameraOrientation;

        void updateOrientation();
        void calculateFront();
        void calculateRight();
    
    protected:
        float fov, aspect, nearPlane, farPlane;
        vec3 cameraPosition, up, front, right;
        float pitch = 0.0f, yaw = 90.0f;
};

#endif