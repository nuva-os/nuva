/*
 * Nuva OS
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/* * Nuva OS POSIX Standard Header
 * Compliant with Standard


 */

#ifndef _POSIX_H
#define _POSIX_H

/* POSIX Version */
#define _POSIX_VERSION      201701L
#define _POSIX2_VERSION     201701L
#define _POSIX_ASYNCHRONOUS_IO 1
#define _POSIX_MEMLOCK      1
#define _POSIX_MEMLOCK_RANGE 1
#define _POSIX_MESSAGE_PASSING 1
#define _POSIX_PRIORITIZED_IO 1
#define _POSIX_PRIORITY_SCHEDULING 1
#define _POSIX_REALTIME_SIGNALS 1
#define _POSIX_SEMAPHORES  1
#define _POSIX_SHARED_MEMORY_OBJECTS 1
#define _POSIX_SYNCHRONIZED_IO 1
#define _POSIX_TIMERS      1

/* SystemLimit */
#define _POSIX_ARG_MAX     4096
#define _POSIX_CHILD_MAX   25
#define _POSIX_LINK_MAX    8
#define _POSIX_MAX_CANON   255
#define _POSIX_MAX_INPUT   255
#define _POSIX_NAME_MAX    14
#define _POSIX_NGROUPS_MAX 8
#define _POSIX_OPEN_MAX    20
#define _POSIX_PATH_MAX    256
#define _POSIX_PIPE_BUF    512
#define _POSIX_RE_DUP_MAX  255
#define _POSIX_RTSIG_MAX   8
#define _POSIX_SEM_NSEMS_MAX 256
#define _POSIX_SEM_VALUE_MAX 32767
#define _POSIX_SIGQUEUE_MAX 32
#define _POSIX_SSIZE_MAX   32767
#define _POSIX_STREAM_MAX  8
#define _POSIX_TZNAME_MAX  6

/* File Mode Types
#define S_IFMT   0170000
#define S_IFDIR  0040000
#define S_IFCHR  0020000
#define S_IFBLK  0060000
#define S_IFREG  0100000
#define S_IFIFO  0010000
#define S_IFLNK  0120000
#define S_IFSOCK 0140000

#define S_ISDIR(m)  (((m) & S_IFMT) == S_IFDIR)
#define S_ISCHR(m)  (((m) & S_IFMT) == S_IFCHR)
#define S_ISBLK(m)  (((m) & S_IFMT) == S_IFBLK)
#define S_ISREG(m)  (((m) & S_IFMT) == S_IFREG)
#define S_ISFIFO(m) (((m) & S_IFMT) == S_IFIFO)
#define S_ISLNK(m)  (((m) & S_IFMT) == S_IFLNK)
#define S_ISSOCK(m) (((m) & S_IFMT) == S_IFSOCK)

/* FilePermission */
#define S_ISUID  04000
#define S_ISGID  02000
#define S_ISVTX  01000
#define S_IRWXU  0700
#define S_IRUSR  0400
#define S_IWUSR  0200
#define S_IXUSR  0100
#define S_IRWXG  0070
#define S_IRGRP  0040
#define S_IWGRP  0020
#define S_IXGRP  0010
#define S_IRWXO  0007
#define S_IROTH  0004
#define S_IWOTH  0002
#define S_IXOTH  0001

/* OpenFlag */
#define O_RDONLY    0
#define O_WRONLY    1
#define O_RDWR      2
#define O_CREAT     0100
#define O_EXCL      0200
#define O_NOCTTY    0400
#define O_TRUNC     01000
#define O_APPEND    02000
#define O_NONBLOCK  04000
#define O_DSYNC     010000
#define O_SYNC      04010000
#define O_RSYNC     04010000
#define O_DIRECTORY 0200000
#define O_NOFOLLOW  0400000
#define O_CLOEXEC   02000000

/* FileLock */
#define F_DUPFD    0
#define F_GETFD    1
#define F_SETFD    2
#define F_GETFL    3
#define F_SETFL    4
#define F_GETLK    5
#define F_SETLK    6
#define F_SETLKW   7
#define F_GETOWN   8
#define F_SETOWN   9

#define FD_CLOEXEC 1

#define F_RDLCK    0
#define F_WRLCK    1
#define F_UNLCK    2

/* whence */
#define SEEK_SET   0
#define SEEK_CUR   1
#define SEEK_END   2

/* Error Codes
#define EPERM      1
#define ENOENT     2
#define ESRCH      3
#define EINTR      4
#define EIO        5
#define ENXIO      6
#define E2BIG      7
#define ENOEXEC    8
#define EBADF      9
#define ECHILD     10
#define EAGAIN     11
#define ENOMEM     12
#define EACCES     13
#define EFAULT     14
#define ENOTBLK    15
#define EBUSY      16
#define EEXIST     17
#define EXDEV      18
#define ENODEV     19
#define ENOTDIR    20
#define EISDIR     21
#define EINVAL     22
#define ENFILE     23
#define EMFILE     24
#define ENOTTY     25
#define ETXTBSY    26
#define EFBIG      27
#define ENOSPC     28
#define ESPIPE     29
#define EROFS      30
#define EMLINK     31
#define EPIPE      32
#define EDOM       33
#define ERANGE     34
#define EDEADLK    35
#define ENAMETOOLONG 36
#define ENOLCK     37
#define ENOSYS     38
#define ENOTEMPTY  39
#define ELOOP      40
#define EWOULDBLOCK EAGAIN
#define ENOMSG     42
#define EIDRM      43
#define ECHRNG     44
#define EL2NSYNC   45
#define EL3HLT     46
#define EL3RST     47
#define ELNRNG     48
#define EUNATCH    49
#define ENOCSI     50
#define EL2HLT     51
#define EBADE      52
#define EBADR      53
#define EXFULL     54
#define ENOANO     55
#define EBADRQC    56
#define EBADSLT    57
#define EDEADLOCK  EDEADLK
#define EBFONT     59
#define ENOSTR     60
#define ENODATA    61
#define ETIME      62
#define ENOSR      63
#define ENONET     64
#define EPROTONOSUPPORT 65
#define EPROTOTYPE 66
#define ENOPROTOOPT 67
#define EOPNOTSUPP 68
#define EPFNOSUPPORT 69
#define EAFNOSUPPORT 70
#define EADDRINUSE 71
#define EADDRNOTAVAIL 72
#define ENETDOWN   73
#define ENETUNREACH 74
#define ENETRESET  75
#define ECONNABORTED 76
#define ECONNRESET 77
#define ENOBUFS    78
#define EISCONN    79
#define ENOTCONN   80
#define ESHUTDOWN  81
#define ETOOMANYREFS 82
#define ETIMEDOUT  83
#define ECONNREFUSED 84
#define EHOSTDOWN  85
#define EHOSTUNREACH 86
#define EALREADY   87
#define EINPROGRESS 88
#define ESTALE     89
#define EUCLEAN    90
#define ENOTNAM    91
#define ENAVAIL    92
#define EISNAM     93
#define EREMOTEIO  94
#define EDQUOT     95
#define ENOMEDIUM  96
#define EMEDIUMTYPE 97

/* Signal */
#define SIGHUP     1
#define SIGINT     2
#define SIGQUIT    3
#define SIGILL     4
#define SIGTRAP    5
#define SIGABRT    6
#define SIGIOT     6
#define SIGBUS     7
#define SIGFPE     8
#define SIGKILL    9
#define SIGUSR1    10
#define SIGSEGV    11
#define SIGUSR2    12
#define SIGPIPE    13
#define SIGALRM    14
#define SIGTERM    15
#define SIGSTKFLT  16
#define SIGCHLD    17
#define SIGCONT    18
#define SIGSTOP    19
#define SIGTSTP    20
#define SIGTTIN    21
#define SIGTTOU    22
#define SIGURG     23
#define SIGXCPU    24
#define SIGXFSZ    25
#define SIGVTALRM  26
#define SIGPROF    27
#define SIGWINCH   28
#define SIGIO      29
#define SIGPOLL    SIGIO
#define SIGPWR     30
#define SIGSYS     31
#define SIGUNUSED  31

#define SIG_DFL    ((void (*)(int))0)
#define SIG_IGN    ((void (*)(int))1)
#define SIG_ERR    ((void (*)(int))-1)

/* Process */
#define WNOHANG    0x00000001
#define WUNTRACED  0x00000002
#define WCONTINUED 0x00000008

#define WIFEXITED(s)   (((s) & 0x7f) == 0)
#define WEXITSTATUS(s) (((s) >> 8) & 0xff)
#define WIFSIGNALED(s) (((s) & 0x7f) != 0 && ((s) & 0x7f) != 0x7f)
#define WTERMSIG(s)    ((s) & 0x7f)
#define WIFSTOPPED(s)  (((s) & 0xff) == 0x7f)
#define WSTOPSIG(s)    (((s) >> 8) & 0xff)
#define WIFCONTINUED(s) ((s) == 0xffff)

#endif /* _POSIX_H */