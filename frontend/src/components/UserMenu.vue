<script setup ts>
import { useAuth } from '@/stores/auth';
const auth = useAuth();
</script>
<template>
    <ul class="navbar-nav ms-auto">
        <li class="nav-item dropdown" id="auth-menu">
            <a class="nav-link dropdown-toggle" href="#" role="button" data-bs-toggle="dropdown" aria-expanded="false">
                <i data-feather="user"></i>
                <span class="user-name">Username</span>
            </a>
            <ul class="dropdown-menu dropdown-menu-end">
                <li><a class="dropdown-item" href="#">Action</a></li>
                <li><a class="dropdown-item" href="#">Another action</a></li>
                <li>
                    <hr class="dropdown-divider">
                </li>
                <li>
                    <a class="dropdown-item" @click="auth.logout()" style="cursor: pointer;">
                        <i data-feather="log-out"></i>
                        Log out
                    </a>
                </li>
            </ul>
        </li>
    </ul>
</template>