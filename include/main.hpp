/**
 * @file main.hpp
 * @author Veer Singh
 * @brief all includes needed for this project
 * @version 0.0.7
 * @date 2024-12-01
 * 
 * @copyright Copyright (c) 2024
 * 
 */

#pragma region std includes
#include <iostream>
#include <vector>
#include <algorithm>
#include <optional>
#include <set>

#ifdef _WIN32
#include <windows.h>
#endif

#pragma region non-std includes
#include <SDL2/SDL.h>
#include <SDL2/SDL_vulkan.h>
#include <vulkan/vulkan.h>

#pragma region custom headers
#include "engine.hpp"

// namespace simplifications

using namespace std;