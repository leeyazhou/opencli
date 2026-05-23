import os
import re
import glob

def update_colors(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Skip files that already have good handling or don't need it (like BaseSwitch)
    if 'BaseSwitch.vue' in file_path or 'BaseSelect.vue' in file_path:
        return

    original = content

    # Replace border colors
    content = re.sub(r'border-white/5', r'border-border', content)
    
    # Text colors
    content = re.sub(r'text-slate-[23]00(?!\s*dark:)', r'text-foreground', content)
    content = re.sub(r'text-slate-[456]00(?!\s*dark:)', r'text-muted-foreground', content)
    content = re.sub(r'text-slate-100(?!\s*dark:)', r'text-foreground', content)
    
    # Backgrounds
    content = re.sub(r'bg-white/5(?!\s*dark:)', r'bg-black/5 dark:bg-white/5', content)
    content = re.sub(r'bg-white/10(?!\s*dark:)', r'bg-black/10 dark:bg-white/10', content)
    content = re.sub(r'bg-black/20(?!\s*dark:)', r'bg-black/5 dark:bg-black/20', content)
    content = re.sub(r'bg-black/40(?!\s*dark:)', r'bg-black/10 dark:bg-black/40', content)
    content = re.sub(r'bg-black/30(?!\s*dark:)', r'bg-black/5 dark:bg-black/30', content)

    # Some specifics for ChatMessageItem
    content = content.replace("bg-[#0f0e1c]/45", "bg-panel")

    if content != original:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Updated {file_path}")

def main():
    base_dir = '/opt/code/ai/opencli/desktop/src/renderer/components'
    for root, dirs, files in os.walk(base_dir):
        for file in files:
            if file.endswith('.vue'):
                update_colors(os.path.join(root, file))
    
    # Also update Chat.vue which is under chat
    chat_file = '/opt/code/ai/opencli/desktop/src/renderer/components/chat/Chat.vue'
    if os.path.exists(chat_file):
        update_colors(chat_file)

if __name__ == '__main__':
    main()
