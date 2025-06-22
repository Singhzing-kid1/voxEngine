#include "main.hpp"

template <typename T> Coord<T>::Coord(){}

template <typename T> Coord<T>& Coord<T>::operator=(const Coord<T>& coord){
    x = coord.x;
    y = coord.y;
    z = coord.z;

    return *this;
}

template <typename T> bool Coord<T>::operator==(const Coord<T>& coord){
    bool result = false;

    if(x == coord.x && y == coord.y && z == coord.z) result = true;

    return result;
}

template <typename T> bool Coord<T>::operator>=(const Coord<T>& coord){
    bool result = false;

    if(x >= coord.x && y >= coord.y && z >= coord.z) result = true;

    return result;
}

template <typename T> bool Coord<T>::operator<=(const Coord<T>& coord){
    bool result = false;

    if(x <= coord.x && y <= coord.y && z <= coord.z) result = true;

    return result;
}

template <typename T> bool Coord<T>::operator<(const Coord<T>& coord) const{
    if (x != coord.x) return x < coord.x;
    if (y != coord.y) return y < coord.y;
    return z < coord.z;
}

template <typename T> bool Coord<T>::operator!=(const Coord<T>& coord){
    bool result = false;

    if(x != coord.x && y != coord.y && z != coord.z) result = true;

    return result;
}

template <typename U> ostream& operator<<(ostream& os, const Coord<U>& coord){
    os << "(" << coord.x << ", " << coord.y << ", " << coord.z << ")";
    return os;
}

template <typename T> Coord<T> Coord<T>::offset(Coord<T> coord){
    Coord<T> _coord = Coord(x + coord.x, y + coord.y, z + coord.z);

    return _coord;
}

template <typename T> Coord<T>::operator vec3() const {
    return vec3(static_cast<float>(x), static_cast<float>(z), static_cast<float>(z));
}

Voxel::Voxel(fCoord position, int value, int worldHeight, PerlinNoise heightMap) : position(position), value(value) {
    determineFaces(heightMap, worldHeight);
}

Voxel::Voxel() {}

void Voxel::determineFaces(PerlinNoise heightMap, int worldHeight){
    int height = worldHeight * heightMap.octave2D_01(position.x * 0.01, position.z * 0.01, 4);
    if(position.y <= height){
        for(auto direction : directions){
            iCoord voxel = position.offset(direction);
            int _height = worldHeight * heightMap.octave2D_01(voxel.x * 0.01, voxel.z * 0.01, 4);
            if(voxel.y >= _height){
                faces.push_back(direction);
                isSurface = true;
            }            
        }
    } else pass;
}

Chunk::Chunk(worldGenInfo genInfo, vec3 origin, int worldHeight) : origin(origin) {
    for(int x = 0; x < CHUNK_SIZE; x++){
        for(int z = 0; z < CHUNK_SIZE; z++){
            grid[x][0][z] = Voxel(fCoord(x, 0, z), 1, worldHeight, genInfo.heightMap);
        }
    }
}

void Chunk::genMesh(){
    Mesh mesh;
    for(auto& plane : grid){
        for(auto& row : plane){
            for(auto& voxel : row){
                if(!voxel.isSurface) continue;
                vec3 voxelCenter = voxel.position;
                for(auto& _face : voxel.faces) {
                    GLuint idxStart = mesh.verts.size();
                    vec3 normal = _face;
                    vector<vec3> face = faces[_face];

                    mesh.normals.insert(mesh.normals.end(), 4, normal);
                    mesh.verts.insert(mesh.verts.end(), {(vec3)voxel.position + (face[0] * 0.5f), (vec3)voxel.position + (face[1] * 0.5f), (vec3)voxel.position + (face[2] * 0.5f), (vec3)voxel.position + (face[3] * 0.5f)});
                    mesh.inds.insert(mesh.inds.end(), {idxStart + 0, idxStart + 1, idxStart + 2, idxStart + 0, idxStart + 3, idxStart + 1});
                    mesh.colors.insert(mesh.colors.end(), 4, vec3(0.7f));
                }
            }
        }
    }

    lods.insert({1, mesh});
}

Mesh Chunk::getMesh(int lod){
    return lods[lod];
}

World::World(float voxelSize, int worldDimension, int worldHeight, int renderDist, String seed) : worldHeight(worldHeight), worldDimension(worldDimension), renderDist(renderDist) {
    cout << "   preSeed\n";
    unsigned int seed1, seed2, seed3;
    int len = seed.length() / 3;
    char buffer[len];

    size_t end = seed.copy(buffer, len, (len * 0));
    buffer[end] = '\0';
    seed1 = hash((String)buffer);

    seed.copy(buffer, len, (len * 1));
    buffer[end] = '\0';
    seed2 = hash((String)buffer);

    seed.copy(buffer, len, (len * 2));
    buffer[end] = '\0';
    seed3 = hash((String)buffer);
    cout << "   reseed\n";
    worldSeed.heightMap.reseed(seed1);
    worldSeed.moistureMap.reseed(seed2);
    worldSeed.tempMap.reseed(seed3);
    cout << "   preChunk\n";
    world.push_back(Chunk(worldSeed, vec3(0, 0, 0), worldHeight));
    cout << "   chunk\n";
    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glGenBuffers(1, &ebo);
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos) {
    shader.use();
    shader.setUniform("view", view);
    shader.setUniform("projection", projection);
    shader.setUniform("model", model);
    shader.setUniform("lightPosition", vec3(0, 5, 0));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(vao);
    glDrawElements(GL_TRIANGLES, indsSize, GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update() {pass;}

void World::prepAndCombineBuffers() {
    vector<vec3> verts, normals, colors;
    vector<unsigned int> inds;
    for(auto chunk : world){
        if(!chunk.buffered) continue;
        Mesh mesh = chunk.getMesh(1);
        verts.insert(verts.end(), mesh.verts.begin(), mesh.verts.end());
        normals.insert(normals.end(), mesh.normals.begin(), mesh.normals.end());
        colors.insert(colors.end(), mesh.colors.begin(), mesh.colors.end());
        inds.insert(inds.end(), mesh.inds.begin(), mesh.inds.end());
    }

    indsSize = inds.size();

    glBindVertexArray(vao);

    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, (verts.size() + normals.size() + colors.size()) * sizeof(vec3), nullptr, GL_DYNAMIC_DRAW);

    glBufferSubData(GL_ARRAY_BUFFER, 0, verts.size() * sizeof(vec3), verts.data());
    glBufferSubData(GL_ARRAY_BUFFER, verts.size() * sizeof(vec3), normals.size() * sizeof(vec3), normals.data());
    glBufferSubData(GL_ARRAY_BUFFER, (verts.size() + normals.size()) * sizeof(vec3), colors.size() * sizeof(vec3), colors.data());

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (verts.size() * sizeof(vec3)));
    glEnableVertexAttribArray(1);

    glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) ((verts.size() + normals.size()) * sizeof(vec3)));
    glEnableVertexAttribArray(2);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, inds.size() * sizeof(unsigned int), inds.data(), GL_DYNAMIC_DRAW);

    glBindVertexArray(0);
}

int World::hash(String str){
    int hash = 0;
    for(auto c : str){
        hash += (int)c;
    }

    return hash;
}