import os
import re

def refine_colors(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Skip files that shouldn't be touched
    if 'BaseSwitch.vue' in file_path or 'BaseSelect.vue' in file_path:
        return

    original = content

    # Remove light mode background colors, replacing them with transparent
    # so we rely purely on borders for separation in light mode
    content = re.sub(r'bg-black/5 dark:bg-black/20', r'bg-transparent dark:bg-black/20', content)
    content = re.sub(r'bg-black/5 dark:bg-white/5', r'bg-transparent dark:bg-white/5', content)
    content = re.sub(r'bg-black/10 dark:bg-white/10', r'bg-transparent dark:bg-white/10', content)
    
    # Specific to Chat.vue
    content = re.sub(r'bg-black/10 flex items-center', r'bg-transparent flex items-center', content)
    
    # Specific to ChatMessageItem.vue
    content = content.replace("'bg-panel p-3", "'bg-transparent dark:bg-panel p-3")

    if content != original:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Refined {file_path}")

def main():
    base_dir = '/opt/code/ai/opencli/desktop/src/renderer/components'
    for root, dirs, files in os.walk(base_dir):
        for file in files:
            if file.endswith('.vue'):
                refine_colors(os.path.join(root, file))

if __name__ == '__main__':
    main()
