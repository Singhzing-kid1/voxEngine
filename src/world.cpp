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

            accessor.setValueOn(voxel, 1.0f);
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
    shader.setUniform("lightPosition", vec3(0, 5, 0));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(region.vao);
    glDrawElements(GL_TRIANGLES, region.inds.size(), GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update(vec3 playerPos){
    genRegionMesh();
    setupMeshBuffers();
}

void World::sync(vec3 playerPos, int time){
    Coord player(playerPos.x, playerPos.y, playerPos.z);
    Coord min(player - Coord(renderDist/2)), max(player + Coord(renderDist/2)); 
    BBoxd regionSelect(min.asVec3d(), max.asVec3d());

    region.regionGrid = tools::clip(*world, regionSelect); 
    
    region.regionPos = Coord(playerPos.x, 0, playerPos.z);
}

World::Mesh World::isSurfaceVoxel(Coord coord, FloatGrid::Accessor accessor){
    Mesh mesh;
    float value = accessor.getValue(coord);

    vector<Coord> directions = {Coord(0, 0, 1),
                                Coord(0, 1, 0),
                                Coord(1, 0, 0),
                                Coord(0, 0, -1),
                                Coord(0, -1, 0),
                                Coord(-1, 0, 0)};

    for(auto direction : directions){
        Coord offset = coord.offsetBy(direction.x(), direction.y(), direction.z());

        if(accessor.getValue(offset) == -1){
            mesh.isSurface == true;
            mesh.directions.push_back(direction);
        }
    }
    

    return mesh;
}

vec3 World::voxelToWorld(Coord coord, Coord regionPos, float voxelSize){
    Vec3i _coord = coord.asVec3i();
    Vec3s _regionPos = regionPos.asVec3s();

    return vec3(_regionPos.x() + _coord.x() * voxelSize, _regionPos.y() + _coord.y() * voxelSize, _regionPos.z() + _coord.z() * voxelSize);
}

void World::genRegionMesh(){
    FloatGrid::Accessor accessor = region.regionGrid->getAccessor();
    vector<vec3> _verts, _norms, _colors;
    vector<unsigned int> _inds;

    for(auto iter = region.regionGrid->beginValueOn(); iter; ++iter){
        Coord voxel = iter.getCoord();
        Mesh mesh = isSurfaceVoxel(voxel, accessor);

        vec3 cyan(0, 1, 1);
        vec3 magenta(1, 0, 1);
        vec3 yellow(1, 1, 0);
        vec3 black(0, 0, 0);

        vec3 voxelCenter = voxelToWorld(voxel, region.regionPos, voxelSize);
        if(mesh.isSurface){
            for(auto direction : mesh.directions){
                GLuint idxStart = _verts.size();
                vec3 normal = vec3(direction.asVec3s().x(), direction.asVec3s().y(), direction.asVec3s().z());
                vector<vec3> face = faces[direction];

                _norms.insert(_norms.end(), {normal, normal, normal, normal});                
                _verts.insert(_verts.end(), {voxelCenter + (face[0] * (voxelSize / 2)), voxelCenter +  (face[1] * (voxelSize / 2)), voxelCenter +  (face[2] * (voxelSize / 2)), voxelCenter +  (face[3] * (voxelSize / 2))});
                _inds.insert(_inds.end(), {idxStart + 0, idxStart + 1, idxStart + 2, idxStart + 0, idxStart + 3, idxStart + 1});

                _colors.insert(_colors.end(), {cyan, magenta, yellow, black});
            }   
        }
    }

    region.verts.assign(_verts.begin(), _verts.end());
    region.inds.assign(_inds.begin(), _inds.end());
    region.normals.assign(_norms.begin(), _norms.end());
    region.colors.assign(_colors.begin(), _colors.end());
}


void World::setupMeshBuffers(){
    glGenVertexArrays(1, &region.vao);
    glGenBuffers(1, &region.vbo);
    glGenBuffers(1, &region.ebo);

    glBindVertexArray(region.vao);

    glBindBuffer(GL_ARRAY_BUFFER, region.vbo);
    glBufferData(GL_ARRAY_BUFFER, (region.verts.size() + region.normals.size() + region.colors.size()) * sizeof(vec3), nullptr, GL_STATIC_DRAW);

    glBufferSubData(GL_ARRAY_BUFFER, 0, region.verts.size() * sizeof(vec3), region.verts.data());

    glBufferSubData(GL_ARRAY_BUFFER, region.verts.size() * sizeof(vec3), region.normals.size() * sizeof(vec3), region.normals.data());

    glBufferSubData(GL_ARRAY_BUFFER, (region.verts.size() + region.normals.size()) * sizeof(vec3), region.colors.size() * sizeof(vec3), region.colors.data());

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) 0);
    glEnableVertexAttribArray(0);

    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) (region.verts.size() * sizeof(vec3)));
    glEnableVertexAttribArray(1);

    glVertexAttribPointer(2, 3, GL_FLOAT, GL_FALSE, 3 * sizeof(float), (void*) ((region.verts.size() + region.normals.size()) * sizeof(vec3)));
    glEnableVertexAttribArray(2);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, region.ebo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, region.inds.size() * sizeof(unsigned int), region.inds.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);
}


