#include "main.hpp"

DebugDrawer::DebugDrawer() : m_debugMode(0), m_vao(0), m_vbo(0) {
    // Load debug line shader

    // Setup VAO/VBO for dynamic line rendering
    glGenVertexArrays(1, &m_vao);
    glGenBuffers(1, &m_vbo);
    glBindVertexArray(m_vao);
    glBindBuffer(GL_ARRAY_BUFFER, m_vbo);
    glEnableVertexAttribArray(0);  // position
    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(glm::vec4), (void*)0);
    glEnableVertexAttribArray(1);  // color
    glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, sizeof(glm::vec4), (void*)(sizeof(glm::vec3)));
    glBindVertexArray(0);
}

DebugDrawer::~DebugDrawer() {
    glDeleteVertexArrays(1, &m_vao);
    glDeleteBuffers(1, &m_vbo);
}

void DebugDrawer::setMatrices(const glm::mat4& model, const glm::mat4& view, const glm::mat4& proj) {
    m_model = model;
    m_view = view;
    m_proj = proj;
}

void DebugDrawer::drawLine(const btVector3& from, const btVector3& to, const btVector3& color) {
    if (!(m_debugMode & btIDebugDraw::DBG_DrawWireframe)) return;
    m_lines.push_back({glm::vec3(from.x(), from.y(), from.z()),
                       glm::vec3(to.x(), to.y(), to.z()),
                       glm::vec3(color.x(), color.y(), color.z())});
}

void DebugDrawer::render(Shader m_shader) {
    if (m_lines.empty() || !m_vao) return;

    // Update VBO with all lines (2 verts per line: from + to)
    std::vector<glm::vec4> vertices;
    vertices.reserve(m_lines.size() * 2);
    for (const auto& line : m_lines) {
        vertices.push_back(glm::vec4(line.from, 1.0f));
        vertices.push_back(glm::vec4(line.to, 1.0f));
    }

    glBindVertexArray(m_vao);
    glBindBuffer(GL_ARRAY_BUFFER, m_vbo);
    glBufferData(GL_ARRAY_BUFFER, vertices.size() * sizeof(glm::vec4), vertices.data(), GL_DYNAMIC_DRAW);

    // Use shader and set matrices
    m_shader.use();
    m_shader.setUniform("model", m_model);
    m_shader.setUniform("view", m_view);
    m_shader.setUniform("projection", m_proj);

    glEnable(GL_DEPTH_TEST);
    glEnable(GL_LINE_SMOOTH);
    glLineWidth(2.0f);
    glDrawArrays(GL_LINES, 0, static_cast<int>(vertices.size()));

    glBindVertexArray(0);
    m_lines.clear();  // Clear for next frame
}