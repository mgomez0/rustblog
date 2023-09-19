document.addEventListener("DOMContentLoaded", () => {
    const postList = document.getElementById("post-list");

    async function fetchPosts() {
        try {
            const response = await fetch("http://localhost:3030/posts");

            if (!response.ok) {
                throw new Error(`Request failed with status: ${response.status}`);
            }

            const posts = await response.json();
            renderPosts(posts);
        } catch (error) {
            console.error("Error:", error);
        }
    }

    function renderPosts(posts) {
        postList.innerHTML = posts.map(post => `
            <div class="post">
                <h2>${post.title}</h2>
                <div class="markdown-content">${parseMarkdown(post.body)}</div>
            </div>
        `).join("");
    }

    function parseMarkdown(markdown) {
        return marked.parse(markdown);
    }

    fetchPosts();
});
