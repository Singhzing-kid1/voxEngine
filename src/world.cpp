#include "main.hpp"

World::World(float voxelSize, int worldDimension, int worldHeight, int renderDist) : worldHeight(worldHeight), voxelSize(voxelSize), renderDist(renderDist) {
    initialize();

    world = FloatGrid::create(-1.0f);
    world->setTransform(math::Transform::createLinearTransform(voxelSize));

    FloatGrid::Accessor accessor = world->getAccessor();

    Coord start(-50, 0, -50);

    for(int x = 0; x < 100; x++){
        for(int z = 0; z < 100; z++){
            Coord voxel = start.offsetBy(x, 0, z);

            accessor.setValue(voxel, 1.0f);
        }
    }

    region.regionGrid = FloatGrid::create(-1.0f);
    region.regionGrid->setTransform(math::Transform::createLinearTransform(voxelSize));
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos){
    shader.use();
    shader.setUniform("view", view);
    shader.setUniform("projection", projection);
    shader.setUniform("model", model);
    shader.setUniform("color", vec3(1.0f, 1.0f, 1.0f));
    shader.setUniform("lightPosition", vec3(8, 5, 5));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(region.vao);
    glDrawElements(GL_TRIANGLES, region.inds.size(), GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update(vec3 playerPos, int time){
    sync(playerPos);

    if(region.updateRegion){
        genRegionMesh();
        setupMeshBuffers();

        region.updateRegion = false;
    }
}

void World::sync(vec3 playerPos){

    if((abs(playerPos.x - region.regionPos.x()) >= (renderDist/4)  || abs(playerPos.z - region.regionPos.z()) >= (renderDist/4) || abs(playerPos.y - region.regionPos.y()) >= (renderDist /4)) || newWorld){
        FloatGrid::Accessor worldAccess = world->getAccessor();
        FloatGrid::Accessor regionAccess = region.regionGrid->getAccessor();

        Coord start(round(playerPos.x) - (renderDist / 2), floor(playerPos.y) - (renderDist / 2), floor(playerPos.z) - (renderDist / 2));
        Coord _start(-(renderDist / 2), -(renderDist / 2), -(renderDist / 2));

        for(int x = 0; x < renderDist; x++){
            for(int y = 0; y < renderDist; y++){
                for(int z = 0; z < renderDist; z++){
                    Coord voxel = start.offsetBy(x, y, z);
                    Coord _voxel = _start.offsetBy(x, y, z);

                    regionAccess.setValue(_voxel, worldAccess.getValue(voxel));
                }
            }
        }

        region.regionPos = Coord(playerPos.x, playerPos.y, playerPos.z);
        region.updateRegion = true;

        newWorld = newWorld ? !newWorld : newWorld;
    }
}

bool World::isSurfaceVoxel(Coord coord, FloatGrid::Ptr grid){
    FloatGrid::Accessor accessor = region.regionGrid->getAccessor();

    float value = accessor.getValue(coord);

    vector<Coord> directions = {Coord(0, 0, 1),
                                Coord(0, 1, 0),
                                Coord(1, 0, 0),
                                Coord(0, 0, -1),
                                Coord(0, -1, 0),
                                Coord(-1, 0, 0)};

    for(auto direction : directions){
        Coord offset;

        if((coord.x() >= 0 || coord.y() >= 0 || coord.z() >= 0)){
            offset = coord.offsetBy(direction.x(), direction.y(), direction.z());

            if(accessor.getValue(offset) == -1){
                return true;
            }
    

        }
    }

    return false;
}

vec3 World::voxelToWorld(Coord coord, Coord regionPos, float voxelSize){
    Vec3i _coord = coord.asVec3i();
    Vec3s _regionPos = regionPos.asVec3s();

    return vec3(_regionPos.x() + _coord.x() * voxelSize, _regionPos.y() + _coord.y() * voxelSize, _regionPos.z() + _coord.z() * voxelSize);
}

void World::genRegionMesh(){
    FloatGrid::Accessor accessor = region.regionGrid->getAccessor();

    vector<vec3> _verts, _normals;
    vector<unsigned int> _inds;

    auto addQuad = [&](const vec3& v0, const vec3& v1, const vec3& v2, const vec3& v3, vec3 normal){
        GLuint idxStart = _verts.size();

        _normals.push_back(normal);
        _normals.push_back(normal);
        _normals.push_back(normal);
        _normals.push_back(normal);

        _verts.push_back(v0);
        _verts.push_back(v1);
        _verts.push_back(v2);
        _verts.push_back(v3);

        _inds.push_back(idxStart + 0);
        _inds.push_back(idxStart + 1);
        _inds.push_back(idxStart + 2);

        _inds.push_back(idxStart + 0);
        _inds.push_back(idxStart + 3);
        _inds.push_back(idxStart + 1);
    };


    for(auto iter = region.regionGrid->beginValueOn(); iter; ++iter){
        const Coord& coord = iter.getCoord();
        if (accessor.getValue(coord) != 1.0f) continue;

        vec3 voxelCenter = voxelToWorld(coord, region.regionPos, voxelSize);


        if(isSurfaceVoxel(coord, region.regionGrid)){
            if (accessor.getValue(coord.offsetBy(1, 0, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + vec3(0.5f, -0.5f, -0.5f),
                    voxelCenter + vec3(0.5f,  0.5f,  0.5f),
                    voxelCenter + vec3(0.5f, -0.5f,  0.5f),
                    voxelCenter + vec3(0.5f,  0.5f, -0.5f),
                    vec3(1.0f, 0.0f, 0.0f)
                );     
            }
            
            if (accessor.getValue(coord.offsetBy(-1, 0, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + vec3(-0.5f, -0.5f, -0.5f),
                    voxelCenter + vec3(-0.5f,  0.5f,  0.5f),
                    voxelCenter + vec3(-0.5f,  0.5f, -0.5f),
                    voxelCenter + vec3(-0.5f, -0.5f,  0.5f),
                    vec3(-1.0f, 0.0f, 0.0f)
                );
            }
            
            if (accessor.getValue(coord.offsetBy(0, 1, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + vec3(-0.5f, 0.5f, -0.5f),
                    voxelCenter + vec3( 0.5f, 0.5f,  0.5f),
                    voxelCenter + vec3( 0.5f, 0.5f, -0.5f),
                    voxelCenter + vec3(-0.5f, 0.5f,  0.5f),
                    vec3(0.0f, 1.0f, 0.0f)
                );
            }

            if (accessor.getValue(coord.offsetBy(0, -1, 0)) == -1.0f) {
                addQuad(
                    voxelCenter + vec3(-0.5f, -0.5f, -0.5f),
                    voxelCenter + vec3( 0.5f, -0.5f,  0.5f),
                    voxelCenter + vec3(-0.5f, -0.5f,  0.5f),
                    voxelCenter + vec3( 0.5f, -0.5f, -0.5f),
                    vec3(0.0f, -1.0f, 0.0f)
                );
            }
            
            if (accessor.getValue(coord.offsetBy(0, 0, 1)) == -1.0f) {
                addQuad(
                    voxelCenter + vec3(-0.5f, -0.5f, 0.5f),
                    voxelCenter + vec3( 0.5f,  0.5f, 0.5f),
                    voxelCenter + vec3(-0.5f,  0.5f, 0.5f),
                    voxelCenter + vec3( 0.5f, -0.5f, 0.5f),
                    vec3(0.0f, 0.0f, -1.0f)
                );
            }

            if (accessor.getValue(coord.offsetBy(0, 0, -1)) == -1.0f) {
                addQuad(
                    voxelCenter + vec3(-0.5f, -0.5f, -0.5f),
                    voxelCenter + vec3( 0.5f,  0.5f, -0.5f),
                    voxelCenter + vec3( 0.5f, -0.5f, -0.5f),
                    voxelCenter + vec3(-0.5f,  0.5f, -0.5f),
                    vec3(0.0f, 0.0f, -1.0f)
                );
            }
        }
    }
    region.verts.assign(_verts.begin(), _verts.end());
    region.inds.assign(_inds.begin(), _inds.end());
    region.normals.assign(_normals.begin(), _normals.end());
}


void World::setupMeshBuffers(){
    glGenVertexArrays(1, &region.vao);
    glGenBuffers(1, &region.vbo);
    glGenBuffers(1, &region.ebo);

    glBindVertexArray(region.vao);

    glBindBuffer(GL_ARRAY_BUFFER, region.vbo);
    glBufferData(GL_ARRAY_BUFFER, region.verts.size() * sizeof(vec3) * 2, nullptr, GL_STATIC_DRAW);

    glBufferSubData(GL_ARRAY_BUFFER, 0, region.verts.size() * sizeof(vec3), region.verts.data());

    glBufferSubData(GL_ARRAY_BUFFER, region.verts.size() * sizeof(vec3), region.normals.size() * sizeof(vec3), region.normals.data());

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (region.verts.size() * sizeof(vec3)));
    glEnableVertexAttribArray(1);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, region.ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, region.inds.size() * sizeof(unsigned int), region.inds.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);
}


