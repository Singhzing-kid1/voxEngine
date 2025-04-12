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

            accessor.setValueOnly(voxel, 1.0f);
        }
    }
}

void World::render(mat4 model, mat4 view, mat4 projection, Shader shader, vec3 cameraPos){
    shader.use();
    shader.setUniform("view", view);
    shader.setUniform("projection", projection);
    shader.setUniform("model", model);
    shader.setUniform("lightPosition", vec3(0, 5, 0));
    shader.setUniform("viewPos", cameraPos);

    glBindVertexArray(vao);
    glDrawElements(GL_TRIANGLES, inds.size(), GL_UNSIGNED_INT, 0);
    glBindVertexArray(0);
}

void World::update(){
    genRegionMesh();
    setupMeshBuffers();
}

void World::sync(vec3 playerPos){
        Coord player(playerPos.x / voxelSize, playerPos.y / voxelSize, playerPos.z / voxelSize);
        Coord min(player - Coord(renderDist/2)), max(player + Coord(renderDist/2));

        CoordBBox bbox(min, max);

        tools::foreach(world->beginValueAll(), [&](const FloatGrid::ValueAllIter& iter){
            if(bbox.isInside(iter.getCoord()) && iter.getValue() != -1.0f){
                iter.setActiveState(true);
            } else if(iter.getValue() != -1.0f) {
                iter.setActiveState(false);
            }
        });
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

vec3 World::voxelToWorld(Coord coord, float voxelSize){
    Vec3i _coord = coord.asVec3i();

    return vec3(_coord.x() * voxelSize, _coord.y() * voxelSize, _coord.z() * voxelSize);
}

void World::genRegionMesh(){
    FloatGrid::Accessor accessor = world->getAccessor();
    vector<vec3> _verts, _norms, _colors;
    vector<unsigned int> _inds;

    for(auto iter = world->beginValueOn(); iter; ++iter){
        Coord voxel = iter.getCoord();
        Mesh mesh = isSurfaceVoxel(voxel, accessor);

        vec3 cyan(0, 1, 1);
        vec3 magenta(1, 0, 1);
        vec3 yellow(1, 1, 0);
        vec3 black(0, 0, 0);

        vec3 voxelCenter = voxelToWorld(voxel, voxelSize);
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

    verts.assign(_verts.begin(), _verts.end());
    inds.assign(_inds.begin(), _inds.end());
    normals.assign(_norms.begin(), _norms.end());
    colors.assign(_colors.begin(), _colors.end());
}


void World::setupMeshBuffers(){
    glGenVertexArrays(1, &vao);
    glGenBuffers(1, &vbo);
    glGenBuffers(1, &ebo);

    glBindVertexArray(vao);

    glBindBuffer(GL_ARRAY_BUFFER, vbo);
    glBufferData(GL_ARRAY_BUFFER, (verts.size() + normals.size() + colors.size()) * sizeof(vec3), nullptr, GL_STATIC_DRAW);

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
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, inds.size() * sizeof(unsigned int), inds.data(), GL_STATIC_DRAW);

    glBindVertexArray(0);
}


