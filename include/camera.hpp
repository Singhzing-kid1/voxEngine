/**
 * @file camera.hpp
 * @author Veer Singh
 * @brief Class definition for a Camera Object
 * @version 0.0.4
 * @date 2024-07-27
 * 
 * @copyright Copyright (c) 2024
 * 
 */

#ifndef CAMERA_HPP
#define CAMERA_HPP

#include "main.hpp"

class Camera{

    public:
        Camera(float, float, float, float, float, vec3);

        enum Axis {NORTH, NORTHEAST, EAST, SOUTHEAST, SOUTH, SOUTHWEST, WEST, NORTHWEST};

        vec3 getFrontVec();
        vec3 getPosVec();
        vec3 getUpVec();

        float getFov();
        float getAspect();
        float getNear();
        float getFar();

        void setPitch(float);
        void setYaw(float);

        /**
         * @brief Set the position of the Camera to a global coordinate.
         * @param position vec3 of the position to set the camera too.
         * 
         * @note USED FOR DEBUGGING, could be used for a fast travel or tp system
         * 
         */
        void setPos(vec3);

        void update();

        void moveCamera(Axis, float);

    private:

        float pitch = 0.0f;
        float yaw = 0.0f;
        float deltaTime = 0.0f;
        float smoothingFactor = 0.1;

        float translateX = 0.0f;
        float translateY = 0.0f; // for gravity / jumping stuff later on
        float translateZ = 0.0f;

        float fov;
        float aspect;
        float nearPlane;
        float farPlane;

        vec3 transVec = vec3(0.0f);
        
        quat cameraOrientation;

        vec3 cameraFront;
        vec3 cameraPos;
        vec3 cameraUp;
        vec3 cameraRight;

        void calculateTranslationVec();

        void updateOrientation();
        void smoothUpdate();

        void calculateFrontVec();
        void calculatePosVec();
        void calculateRightVec();



};

#endif