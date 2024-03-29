# 说明

检测目标文件夹下所有的 `git` 仓库，获取master分支最新的`git tag`的信息，根据用户设置的规则，一次性对所有非最新`tag`的仓库(检测master分支)打上新`tag`并推送。
同时返回所有修改记录。

在未有新`commit`的情况下程序返回目前最新`tag`。

# Windows使用说明

1. 下载 `bin`目录下的exe文件和`user.default.toml`文件
2. 配置 `环境变量` (可选)
3. 修改 `user.default.toml`文件名，修改为 `user.toml`
4. 修改 `user.toml` 文件中`huawei`段的账户信息
4. 执行 `git_release.exe`即可

如果没有配置环境变量，需要把exe放置到文件夹内执行，或者通过`--path`指定工作目录

如果配置了环境变量，可以直接在任意目录执行`git_release.exe`，会自动获取当前目录

# MacOS使用说明

类似Windows使用说明

# 华为云API调用顺序
1. KeystoneValidateToken //校验TOKEN是否有效
2. KeystoneCreateUserTokenByPassword // IAM用户登录获取TOKEN
3. ShowJobListByProjectId //查询任务列表
4. RunJob // 执行任务
6. ShowHistoryDetails // 查看任务结果