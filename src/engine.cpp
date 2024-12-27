/**
 * @file engine.cpp
 * @author Veer Singh
 * @brief main engine class; handles window creation/deletion, vulkan setup and takedown and input handling
 * @version 0.0.7
 * @date 2024-12-22
 * 
 * @copyright Copyright (c) 2024
 * 
 */
#include "main.hpp"


Engine::Engine(int width, int height, const char* title){
    this->width = width;
    this->height = height;

    if(SDL_Init(SDL_INIT_VIDEO) < 0) cout << "could not init SDL2. Error: " << SDL_GetError();

    window = SDL_CreateWindow(title, SDL_WINDOWPOS_UNDEFINED, SDL_WINDOWPOS_UNDEFINED, width, height, SDL_WINDOW_VULKAN | SDL_WINDOW_SHOWN);

    if(window == nullptr) cout << "could not create SDL2 window. Error: " << SDL_GetError();

    createInstance();
    createSurface();
    pickPhysicalDevice();
    createLogicalDevice();
}

Engine::~Engine(){
    vkDestroyDevice(device, nullptr);
    vkDestroySurfaceKHR(instance, surface, nullptr);
    vkDestroyInstance(instance, nullptr);
    SDL_DestroyWindow(window);
    SDL_Quit();
}

void Engine::eventHandling(InputData* data){
    while(SDL_PollEvent(&e)){
        switch(e.type){
            case SDL_QUIT:
                data->shouldQuit = true;
                break;

            case SDL_KEYDOWN:
                if(e.key.keysym.sym == SDLK_ESCAPE) data->shouldQuit = true;
                break;
        }
    }
}

void Engine::createInstance(){
    VkApplicationInfo appInfo{};
    appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    appInfo.pApplicationName = "voxEngine test 0.0.7";
    appInfo.applicationVersion = VK_MAKE_VERSION(0, 0, 7);
    appInfo.pEngineName = "voxEngine";
    appInfo.engineVersion = VK_MAKE_VERSION(0, 0, 7);
    appInfo.apiVersion = VK_API_VERSION_1_0;

    VkInstanceCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    createInfo.pApplicationInfo = &appInfo;

    uint32_t extensionCount;
    SDL_Vulkan_GetInstanceExtensions(window, &extensionCount, nullptr);
    vector<const char*> extensions(extensionCount);
    SDL_Vulkan_GetInstanceExtensions(window, &extensionCount, extensions.data());

    createInfo.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
    createInfo.ppEnabledExtensionNames = extensions.data();

    if(vkCreateInstance(&createInfo, nullptr, &instance) != VK_SUCCESS) cout << "failed to creat vulkan instance"; 

}

void Engine::createSurface(){
    if(!SDL_Vulkan_CreateSurface(window, instance, &surface)) cout << "failed to create vulkan surface";
}

void Engine::pickPhysicalDevice(){
    uint32_t deviceCount = 0;
    vkEnumeratePhysicalDevices(instance, &deviceCount, nullptr);

    if(deviceCount == 0) cout << "no GPU's with vulkan support";

    vector<VkPhysicalDevice> devices(deviceCount);
    vkEnumeratePhysicalDevices(instance, &deviceCount, devices.data()); 

    for(const auto& device : devices){
        if(isSuitableDevice(device)){
            pDevice = device;
            break;
        }
    }

    if(pDevice == VK_NULL_HANDLE){
        cout << "failed to find suitable GPU";
    }
}

void Engine::createLogicalDevice(){
    VkDeviceQueueCreateInfo queueCreateInfo{};
    queueCreateInfo.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    queueCreateInfo.queueFamilyIndex = 0;
    queueCreateInfo.queueCount = 1;
    
    float queuePriority = 1.0f;
    queueCreateInfo.pQueuePriorities = &queuePriority;

    VkPhysicalDeviceFeatures deviceFeatures{};

    VkDeviceCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    createInfo.pQueueCreateInfos = &queueCreateInfo;
    createInfo.queueCreateInfoCount = 1;
    createInfo.pEnabledFeatures = &deviceFeatures;

    if(vkCreateDevice(pDevice, &createInfo, nullptr, &device) != VK_SUCCESS){
        cout << "failed to create logical device";
    }

    vkGetDeviceQueue(device, 0, 0, &queue);    
}

bool Engine::isSuitableDevice(VkPhysicalDevice device){
    QueueFamilyIndices indices = findQueueFamilyIndices(device);

    bool extensionsSupported = checkDeviceExtensionSupport(device);

    bool swapChainAdequate = false;
    if(extensionsSupported){
        SwapChainSupportDetails swapChainSupport = querySwapChainSupport(device);
        swapChainAdequate = !swapChainSupport.formats.empty() && !swapChainSupport.presentModes.empty();
    }

    VkPhysicalDeviceFeatures supportedFeatures;
    vkGetPhysicalDeviceFeatures(device, &supportedFeatures);

    return indices.isComplete() && extensionsSupported && swapChainAdequate && supportedFeatures.samplerAnisotropy;
}

QueueFamilyIndices Engine::findQueueFamilyIndices(VkPhysicalDevice device){
    QueueFamilyIndices indices;

    uint32_t queueFamilyCount = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(device, &queueFamilyCount, nullptr);
    vector<VkQueueFamilyProperties> queueFamilies(queueFamilyCount);
    vkGetPhysicalDeviceQueueFamilyProperties(device, &queueFamilyCount, queueFamilies.data());

    int i = 0;

    for(const auto& queueFamily : queueFamilies) {
        if(queueFamily.queueFlags & VK_QUEUE_GRAPHICS_BIT){
            indices.graphicsFamily = i;
        }

        VkBool32 presentSupport = false;
        vkGetPhysicalDeviceSurfaceSupportKHR(device, i, surface, &presentSupport);

        if(presentSupport){
            indices.presentFamily = i;
        }

        if(indices.isComplete()){
            break;
        }

        i++;
    }

    return indices;
}

bool Engine::checkDeviceExtensionSupport(VkPhysicalDevice device){
    uint32_t extensionCount;
    vkEnumerateDeviceExtensionProperties(device, nullptr, &extensionCount, nullptr);
    vector<VkExtensionProperties> aExtensions(extensionCount);
    vkEnumerateDeviceExtensionProperties(device, nullptr, &extensionCount, aExtensions.data());

    set<string> extensions(deviceExtensions.begin(), deviceExtensions.end());

    for(const auto& extension : aExtensions){
        extensions.erase(extension.extensionName);
    }

    return extensions.empty();
}

SwapChainSupportDetails Engine::querySwapChainSupport(VkPhysicalDevice device){
    SwapChainSupportDetails details;

    vkGetPhysicalDeviceSurfaceCapabilitiesKHR(device, surface, &details.capabilities);

    uint32_t formatCount;
    vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, &formatCount, nullptr);

    if (formatCount != 0){
        details.formats.resize(formatCount);
        vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, &formatCount, details.formats.data());
    }

    uint32_t presentCount;
    vkGetPhysicalDeviceSurfacePresentModesKHR(device, surface, &presentCount, nullptr);

    if(presentCount != 0){
        details.presentModes.resize(presentCount);
        vkGetPhysicalDeviceSurfacePresentModesKHR(device, surface, &presentCount, details.presentModes.data());
    }

    return details;   
}