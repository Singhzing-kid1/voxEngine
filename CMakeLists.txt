cmake_minimum_required(VERSION 3.8)

project(insanityGame)

set(OpenGL_GL_PREFERENCE GLVND)

# Set the include directories
include_directories(include)

# Add all header files in the include directory
file(GLOB HEADER_FILES "./include/*.hpp")

# Add all source files in the src directory
file(GLOB SOURCE_FILES "./src/*.cpp")

# Add executable target
add_executable(main ${HEADER_FILES} ${SOURCE_FILES})

#libraries:

#glm
find_package(glm CONFIG REQUIRED)
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