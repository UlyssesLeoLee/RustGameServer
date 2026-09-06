%% test_add.erl — Erlang NIF 端测试 (per task brief step 5)
%%
%% 调用方式 (Phase 1.5 实装后, Erlang 26 + rustler 0.36):
%%   1. erlc test_add.erl
%%   2. 加载 NIF: erl -noshell -eval '
%%        code:add_path("."),  %% NIF .so 路径
%%        rgs_nif:start_link()  %% 显式 load NIF (per rustler::init! load hook)
%%      '
%%   3. 跑 test: erl -noshell -eval "test_add:run()"
%%
%% 当前 W13 PoC: 仅编译, 不跑 NIF (待 Phase 1.5 装 Erlang 26)

-module(test_add).
-export([run/0, test_add_2_3/0, test_add_100_200/0, test_add_negative/0,
         test_echo/0, test_bridge_route_player/0, test_bridge_route_battle/0,
         test_version/0]).

%% 主入口: 跑全部 7 测试
run() ->
    Tests = [
        {test_add_2_3, fun test_add_2_3/0},
        {test_add_100_200, fun test_add_100_200/0},
        {test_add_negative, fun test_add_negative/0},
        {test_echo, fun test_echo/0},
        {test_bridge_route_player, fun test_bridge_route_player/0},
        {test_bridge_route_battle, fun test_bridge_route_battle/0},
        {test_version, fun test_version/0}
    ],
    Results = [run_one(Name, F) || {Name, F} <- Tests],
    Pass = length([ok || {ok, _} <- Results]),
    Fail = length([fail || {fail, _} <- Results]),
    io:format("~n=== test_add summary: pass=~p fail=~p ===~n", [Pass, Fail]),
    case Fail of
        0 -> halt(0);
        _ -> halt(1)
    end.

run_one(Name, F) ->
    try
        F(),
        io:format("[PASS] ~p~n", [Name]),
        {ok, Name}
    catch
        Error:Reason:Stack ->
            io:format("[FAIL] ~p error=~p reason=~p~n  stack=~p~n",
                      [Name, Error, Reason, Stack]),
            {fail, Name}
    end.

%% ====================================================================
%% 测试用例 (与 crates/network-gateway/src/nif_demo.rs::tests 同步)
%% ====================================================================

%% 最简: add(2, 3) = 5
test_add_2_3() ->
    5 = rgs_nif:add(2, 3).

%% 大数: add(100, 200) = 300
test_add_100_200() ->
    300 = rgs_nif:add(100, 200).

%% 负数: add(-5, 5) = 0
test_add_negative() ->
    0 = rgs_nif:add(-5, 5).

%% echo: 返回 {Value, "echoed"} (待 Phase 1.5 实现 binary 类型, 当前 stub)
test_echo() ->
    {42, <<"echoed">>} = rgs_nif:echo(42).

%% 路由: 10101 -> player.v1.PlayerService
test_bridge_route_player() ->
    {"player.v1.PlayerService", "CreateOrUpdate", 0} = rgs_nif:bridge_route(10101).

%% 路由: 20001 -> battle.v1.BattleService
test_bridge_route_battle() ->
    {"battle.v1.BattleService", "BattleAction", 0} = rgs_nif:bridge_route(20001).

%% 版本: 返回 {"rgs_nif", "0.1.0-w13", "nif_version_2_15"}
test_version() ->
    {"rgs_nif", "0.1.0-w13", "nif_version_2_15"} = rgs_nif:version().
