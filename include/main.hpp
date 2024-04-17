#include <iostream>
#include <cmath>
#include <SDL2/SDL.h>
#include <GL/glew.h>
#include <GL/gl.h>
#include <SDL2/SDL_opengl.h>
#include <glm/glm.hpp>
#define GLM_ENABLE_EXPERIMENTAL
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <glm/gtx/string_cast.hpp>
#include <algorithm>
#include <vector>
#include <sstream>
#include <fstream>
#include <string>

#ifdef _WIN32
#include <windows.h>
#endif

#define HEIGHT 1920
#define WIDTH 1080

#include "shaders.hpp"
#include "camera.hpp"
#include "model.hpp"

using namespace std;
using namespace glm;
