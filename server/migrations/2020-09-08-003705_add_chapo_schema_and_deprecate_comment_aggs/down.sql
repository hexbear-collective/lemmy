drop schema chapo cascade;

create trigger refresh_comment
    after insert or delete or update
    on public.comment
    for each row
    execute procedure public.refresh_comment();
  
create trigger refresh_comment_like
    after insert or delete
    on public.comment_like
    for each row
    execute procedure public.refresh_comment_like();

create trigger refresh_user
    after insert or delete or update
    on public.user
    for each row
    execute procedure public.refresh_user();

create trigger refresh_community_user_ban
    after insert or delete
    on public.community_user_ban
    for each row
    execute procedure public.refresh_community_user_ban();

-- Rebuild agg table
insert into comment_aggregates_fast
select * from comment_aggregates_view
on conflict (id) do update 
set score = EXCLUDED.score, upvotes = EXCLUDED.upvotes, 
downvotes = EXCLUDED.downvotes, hot_rank = EXCLUDED.hot_rank, 
hot_rank_active = EXCLUDED.hot_rank_active;