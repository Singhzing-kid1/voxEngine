#ifndef CAMERA_HPP
#define CAMERA_HPP

#include "main.hpp"

class Camera{

    public:
        Camera(float, float, float, float, float, vec3);

        vec3 getFrontVec();
        vec3 getPosVec();
        vec3 getUpVec();

        float getFov();
        float getAspect();
        float getNear();
        float getFar();

        void setPitch(float);
        void setYaw(float);

        void update();

        void moveCamera(string, float);

    private:

        float pitch = 0.0f;
        float yaw = 0.0f;

        float translateX = 0.0f;
        float translateY = 0.0f; // for gravity / jumping stuff later on
        float translateZ = 0.0f;

        float fov;
        float aspect;
        float nearPlane;
        float farPlane;

        vec3 transVec = vec3(0.0f);

        vec3 cameraFront;
        vec3 cameraPos;
        vec3 cameraUp;
        vec3 cameraRight;

        void calculateTranslationVec();

        void calculateFrontVec();
        void calculatePosVec();
        void calculateRightVec();



};

#endif