#include "main.hpp"


UI::UI(int width, int height) : width(width), height(height){
    projection = ortho(0.0f, (float)width, 0.0f, (float)height);
}


void UI::render(Shader shader){
    for(auto element : elements){
        glDisable(GL_DEPTH_TEST);
        unsigned int texture;
        glGenTextures(1, &texture);
        glBindTexture(GL_TEXTURE_2D, texture);

        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);	
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

        GLenum texFormat = GL_BGRA;
        GLint internalFormat = GL_RGBA8;

        glPixelStorei(GL_UNPACK_ALIGNMENT, element.surface->format->BytesPerPixel);
        glPixelStorei(GL_UNPACK_ROW_LENGTH, element.surface->pitch / element.surface->format->BytesPerPixel);

        glTexImage2D(GL_TEXTURE_2D, 0, internalFormat, element.surface->w, element.surface->h, 0, texFormat, GL_UNSIGNED_BYTE, element.surface->pixels);
        shader.use();
        shader.setUniform("projection", projection);

        glBindVertexArray(element.vao);
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture);
        glDrawElements(GL_TRIANGLES, element.inds.size(), GL_UNSIGNED_INT, 0);
        glBindVertexArray(0);
    }
}

int UI::addElement(ElementType type, std::string text, TTF_Font* font, SDL_Color color, int x, int y){
    int id = elements.size() + 1;
    uiElement element;
    element.surface = TTF_RenderUTF8_Blended(font, text.c_str(), color);
    if(!element.surface) cout << TTF_GetError();    
    element.type = type;
    element.pos = vec2(x, y);
    element.ID = id;
    
    glGenVertexArrays(1, &element.vao);
    glGenBuffers(1, &element.vbo);
    glGenBuffers(1, &element.ebo);

    elements.push_back(element);
    return id;
}

int UI::addElement(ElementType type, int x, int y){
    pass;
    return 0;
}

void UI::editElement(int id, vec2 pos, TTF_Font* font, SDL_Color color, std::string text = ""){
    elements[id - 1].surface = TTF_RenderUTF8_Blended(font, text.c_str(), color);
    elements[id - 1].pos = pos;
}

void UI::update(){
    genMesh();
    updateBuffers();
}

void UI::genMesh(){
    for(auto element : elements){
        vector<vec2> _verts, _texs;
        vector<unsigned int> _inds;

        GLuint idxStart = _verts.size();;

        int w = element.surface->w , h = element.surface->h;
        
        vec2 origin = vec2(element.pos.x, height  - element.pos.y);
        vec2 tRight = vec2(origin.x + w, origin.y);
        vec2 bRight = vec2(origin.x + w, origin.y - h);
        vec2 bLeft = vec2(origin.x, origin.y - h);

        _verts.insert(_verts.end(), {origin, bLeft, tRight, bRight});
        _texs.insert(_texs.end(), {vec2(0,0), vec2(0,1), vec2(1,0), vec2(1,1)});
        _inds.insert(_inds.end(), {idxStart + 0, idxStart + 1, idxStart + 2, idxStart + 1, idxStart + 3, idxStart + 2});

        elements[element.ID - 1].verts.assign(_verts.begin(), _verts.end());
        elements[element.ID - 1].texs.assign(_texs.begin(), _texs.end());
        elements[element.ID - 1].inds.assign(_inds.begin(), _inds.end());
    }

}

void UI::updateBuffers(){
    for(auto element : elements){
        glBindVertexArray(element.vao);

        glBindBuffer(GL_ARRAY_BUFFER, element.vbo);
        glBufferData(GL_ARRAY_BUFFER, (element.verts.size() + element.texs.size()) * sizeof(vec2), nullptr, GL_DYNAMIC_DRAW);

        glBufferSubData(GL_ARRAY_BUFFER, 0, element.verts.size() * sizeof(vec2), element.verts.data());
        glBufferSubData(GL_ARRAY_BUFFER, element.verts.size() * sizeof(vec2), element.texs.size() * sizeof(vec2), element.texs.data());

        glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), (void*) 0);
        glEnableVertexAttribArray(0);

        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 2 * sizeof(float), (void*) (element.verts.size() * sizeof(vec2)));
        glEnableVertexAttribArray(1);

        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, element.ebo);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, element.inds.size() * sizeof(unsigned int), element.inds.data(), GL_DYNAMIC_DRAW);

        glBindVertexArray(0);
    }
}