#ifndef CAMERA_HPP
#define CAMERA_HPP

#include "main.hpp"

class Camera{
    public:
        Camera(float, float, float, float, float, vec3);

        enum Axis {NORTH, NORTHEAST, EAST, SOUTHEAST, SOUTH, SOUTHWEST, WEST, NORTHWEST};

        void update(float);

        void moveCamera(Axis, float);

        float getItem(float);
        vec3 getItem(int);

        void setAngles(float, float);

    private:
        float fov, aspect, nearPlane, farPlane, deltaTime;
        // #    1,   2,      3,         4  <-- floats;

        quat cameraOrientation;

        void updateOrientation();
        void calculateFront();
        void calculateRight();
        void calculatePos();
    
    protected:
        vec3 pos, up, front, right, translationVec;
        //   1,   2,   3,     4,     5 <-- ints;
        float pitch = 0.0f, yaw = 90.0f;
};

#endif