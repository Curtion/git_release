# 说明

检测目标文件夹下所有的 `git` 仓库，获取最新的`git tag`的信息，根据用户设置的规则，一次性对所有非最新`tag`的仓库打上新`tag`并推送。
同时返回所有修改记录。

在未有新`commit`的情况下程序返回目前最新`tag`。

# 使用说明

1. 下载 `bin`目录下的exe文件
2. 配置 `环境变量` (可选)
3. 执行 `git_release.exe`即可。

如果没有配置环境变量，需要把exe放置到文件夹内执行，或者通过`--path`指定工作目录

如果配置了环境变量，可以直接在任意目录执行`git_release.exe`，会自动获取当前目录

# 华为云API调用顺序
1. KeystoneValidateToken //校验TOKEN是否有效
2. KeystoneCreateUserTokenByPassword // IAM用户登录获取TOKEN
3. ShowJobListByProjectId //查询任务列表
4. RunJob // 执行任务
6. ShowHistoryDetails // 查看任务结果