/**
 * @file main.hpp
 * @author Veer Singh
 * @brief all includes need for this project.
 * @version 0.0.6
 * @date 2024-09-02
 * 
 * @copyright Copyright (c) 2024
 * 
 */


#pragma region std includes 
#include <iostream>
#include <fstream>
#include <sstream>
#include <string>
#include <cmath>
#include <algorithm>
#include <vector>
#include <time.h>
#include <queue>
#include <future>
#include <random>
#include <map>
#include <unordered_map>
#include <thread>
#include <mutex>
#include <atomic>
#include <condition_variable>

#ifdef _WIN32
#include <windows.h>
#endif
#pragma endregion

#pragma region non-std includes
#include <SDL2/SDL.h>
#include <GL/glew.h>
#include <GL/gl.h>
#include <SDL2/SDL_opengl.h>
#include <SDL2/SDL_ttf.h>
#include <SDL2/SDL_pixels.h>
#define GLM_FORCE_SWIZZLE
#include <glm/glm.hpp>
#define GLM_ENABLE_EXPERIMENTAL

#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <glm/gtx/string_cast.hpp>
#include <glm/ext/vector_relational.hpp>
#include <glm/gtc/quaternion.hpp>
#include <glm/gtx/quaternion.hpp>
#include <btBulletCollisionCommon.h>

#pragma endregion

#pragma region custom headers
#include "engine.hpp"
#include "player.hpp"
#include "camera.hpp"
#include "gpuCompute.hpp"
#include "shader.hpp"
#include "world.hpp"
#include "perlin.hpp"
#include "ui.hpp"
#pragma endregion

#define pass (void)0;
#define String std::string

// namespace simplifications

using namespace std;
using namespace glm;
using namespace siv;


