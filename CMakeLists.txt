cmake_minimum_required(VERSION 3.8)

project(insanityGame)

set(OpenGL_GL_PREFERENCE GLVND)

# Set the include directories
include_directories(include)

# Add all header files in the include directory
file(GLOB HEADER_FILES "./include/*.hpp")

# Add all source files in the src directory
file(GLOB SOURCE_FILES "./src/*.cpp")

# Add all shader files
file(GLOB SHADER_SOURCE_FILES "./shaders/*.glsl")
file(COPY ${SHADER_SOURCE_FILES} DESTINATION "shaders")

#add shape/model files
file(GLOB MODEL_SOURCE_FILES "./models/*.model")
file(GLOB SHAPE_FILES "./models/*.shape")

file(COPY ${MODEL_SOURCE_FILES} DESTINATION "models")
file(COPY ${SHAPE_FILES} DESTINATION "models")

# Add executable target
add_executable(main ${HEADER_FILES} ${SOURCE_FILES})

#libraries:

#glm
find_package(glm REQUIRED)
target_link_libraries(main PRIVATE glm::glm)


# gl
find_package(OpenGL REQUIRED)
target_link_libraries(main PRIVATE OpenGL::GL)

#glew
find_package(GLEW REQUIRED)
target_link_libraries(main PRIVATE GLEW::GLEW)

#sdl2
find_package(SDL2 CONFIG REQUIRED)
target_link_libraries(main
    PRIVATE
    $<TARGET_NAME_IF_EXISTS:SDL2::SDL2main>
    $<IF:$<TARGET_EXISTS:SDL2::SDL2>,SDL2::SDL2,SDL2::SDL2-static>
)

find_package(Bullet REQUIRED)
target_link_libraries(main PRIVATE ${BULLET_LIBRARIES})
target_include_directories(main PRIVATE ${BULLET_INCLUDE_DIRS})

# Enable AddressSanitizer
# if (CMAKE_CXX_COMPILER_ID MATCHES "Clang" OR CMAKE_CXX_COMPILER_ID MATCHES "GNU")
#     set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address -fno-omit-frame-pointer -g")
#     set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fsanitize=address -fno-omit-frame-pointer -g")
#     set(CMAKE_LINKER_FLAGS "${CMAKE_LINKER_FLAGS} -fsanitize=address")
# elseif (MSVC)
#     message(WARNING "AddressSanitizer is not supported by MSVC. Consider using Clang or GCC.")
# endif()