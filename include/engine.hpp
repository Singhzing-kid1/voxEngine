/**
 * @file engine.hpp
 * @author Veer Singh
 * @brief handles window and vulkan instance creation and input handling
 * @version 0.0.7
 * @date 2024-12-01
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#ifndef ENGINE_HPP
#define ENGINE_HPP

#include "main.hpp"

struct QueueFamilyIndices {
    optional<uint32_t> graphicsFamily;
    optional<uint32_t> presentFamily;

    bool isComplete() {
        return graphicsFamily.has_value() && presentFamily.has_value();
    }
};

struct SwapChainSupportDetails {
    VkSurfaceCapabilitiesKHR capabilities;
    vector<VkSurfaceFormatKHR> formats;
    vector<VkPresentModeKHR> presentModes;
};

struct InputData {
    float xOffset = 0.0f, yOffset = 0.0f;
    bool shouldQuit = false;
    const Uint8* state;
};

class Engine{
    public:
        Engine(int, int, const char*);
        ~Engine();

        void eventHandling(InputData*);

        int width, height;

    private:
        SDL_Event e;

        SDL_Window* window;

        VkInstance instance;
        VkSurfaceKHR surface;
        VkPhysicalDevice pDevice;
        VkDevice device;
        VkQueue queue;

        const vector<const char*> deviceExtensions = {VK_KHR_SWAPCHAIN_EXTENSION_NAME};

        void createInstance();
        void createSurface();
        void pickPhysicalDevice();
        void createLogicalDevice();

        bool isSuitableDevice(VkPhysicalDevice);
        bool checkDeviceExtensionSupport(VkPhysicalDevice);
        QueueFamilyIndices findQueueFamilyIndices(VkPhysicalDevice);
        SwapChainSupportDetails querySwapChainSupport(VkPhysicalDevice);


        

};

#endif
